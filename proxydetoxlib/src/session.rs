pub mod builder;
pub mod peer;
pub mod shared;

use std::fmt::Write;
use std::fs::read_to_string;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{self, Poll};
use std::time::Duration;

use detox_net::PathOrUri;
use http::header::CONTENT_TYPE;
use http::Response;
use http::Uri;
use hyper::Body;
use paclib::ProxyOrDirect;
use tokio::time::timeout;
use tower::Service;
use tracing_futures::Instrument;

use builder::Builder;
use paclib::proxy::Proxy;
use peer::PeerSession;
use shared::Shared;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid URI")]
    InvalidUri,
    #[error("invalid host: {0}")]
    InvalidHost(
        #[source]
        #[from]
        detox_net::host_and_port::Error,
    ),
    #[error("timeout when connecting to {1} via proxy {0}")]
    ConnectTimeout(Proxy, Uri),
    #[error("upstream error reaching {2} via {1}: {0}")]
    Upstream(#[source] crate::client::Error, Proxy, Uri),
    #[error("error creating client for {1}: {0}")]
    MakeClient(#[source] hyper::Error, Uri),
    #[error("error creating proxy for {1}: {0}")]
    MakeProxyClient(#[source] crate::client::Error, Proxy),
    #[error("client error: {0}")]
    Client(
        #[from]
        #[source]
        hyper::Error,
    ),
    #[error("connect error reaching {1}: {0}")]
    Connect(#[source] tokio::io::Error, Uri),
    #[error("proxy connect error reaching {2} via {1}: {0}")]
    ProxyConnect(#[source] crate::client::ConnectError, Proxy, Uri),
    #[error("upstream proxy ({0}) requires authentication")]
    ProxyAuthenticationRequired(ProxyOrDirect),
    #[error("http error: {0}")]
    Http(
        #[source]
        #[from]
        http::Error,
    ),
    #[error("unable to establish connection: {0}")]
    UnableToEstablishConnection(Uri),
    #[error("handshake error")]
    Handshake,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Session(Arc<Shared>);

impl Session {
    pub fn builder() -> Builder {
        Default::default()
    }

    pub async fn pac_file(
        &self,
        uri: &Option<PathOrUri>,
    ) -> std::result::Result<(), paclib::PacScriptError> {
        tracing::info!("update PAC script");
        let pac = if let Some(uri) = uri {
            let pac = match uri {
                PathOrUri::Path(p) => read_to_string(p)?,
                PathOrUri::Uri(u) => {
                    let pac = timeout(
                        Duration::from_secs(30),
                        crate::net::http_file(u.clone(), (*self.0.tls_config).clone()),
                    )
                    .await;
                    let pac = match pac {
                        Ok(pac) => pac,
                        Err(_) => Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("timeout loading '{}'", &u),
                        )),
                    };
                    pac?
                }
            };
            Some(pac)
        } else {
            None
        };
        self.0.eval.set_pac_script(pac).await
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session").finish()
    }
}

impl<'a> Service<&'a hyper::server::conn::AddrStream> for Session {
    type Response = PeerSession;
    type Error = std::convert::Infallible;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut task::Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, socket: &hyper::server::conn::AddrStream) -> Self::Future {
        let shared = self.0.clone();
        let addr = socket.remote_addr();
        let res = async move {
            tracing::debug!("new connection");
            Ok(PeerSession {
                peer: Arc::new(addr),
                shared,
            })
        };
        let res = res.instrument(tracing::info_span!("call", client_addr=%addr));
        Box::pin(res)
    }
}

fn make_error_html(status: http::StatusCode, message: impl AsRef<str>) -> Result<Response<Body>> {
    let body = format!(
            "<!DOCTYPE html><html><head><title>Error: {}</title></heade><body><h1>Error: {}</h1><p>{}</p><hr><small>{}/{}</small></body></html>",
            status.as_str(),
            status.as_str(),
            message.as_ref(),
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        );
    let resp = Response::builder()
        .status(status)
        .header(
            CONTENT_TYPE,
            http::header::HeaderValue::from_static("text/html"),
        )
        .body(Body::from(body))?;
    Ok(resp)
}

fn make_error_response<E>(error: &E) -> Response<Body>
where
    E: std::error::Error + Send + Sync,
{
    let mut description = String::new();
    write!(&mut description, "<p><strong>Error:</strong> {error}</p>").ok();
    if let Some(cause) = error.source() {
        description
            .write_str("<p><strong>Caused by:</strong></p><ol reversed>")
            .ok();
        for msg in std::iter::successors(Some(cause), |e| e.source()) {
            write!(&mut description, "<li>{msg}</li>").ok();
        }
        description.write_str("</ol>").ok();
    }

    let body = format!(
        include_str!("502.html"),
        description,
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    Response::builder()
        .status(http::StatusCode::BAD_GATEWAY)
        .header(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_static("text/html"),
        )
        .header(http::header::CONNECTION, "close")
        .body(Body::from(body))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::make_error_response;

    #[test]
    fn test_error_response() {
        let resp = make_error_response(&super::Error::InvalidUri);
        assert_ne!(resp.status(), http::StatusCode::OK);
    }
}
