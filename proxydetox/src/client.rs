use http::{Request, Response, StatusCode};
use hyper::Body;
use parking_lot::Mutex;
use proxy_client::HttpProxyInfo;
use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::io::copy_bidirectional;
use tracing_futures::Instrument;

use crate::auth::Authenticator;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP connection upgrade with {1} failed: {0}")]
    UpgradeFailed(#[source] hyper::Error, http::Uri),
    #[error("HTTP CONNECT failed with {0}")]
    ConnectFailed(http::Uri),
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("authentication mechanism error: {0}")]
    Auth(#[from] crate::auth::Error),
    #[error("invalid URI")]
    InvalidUri,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct ProxyClient(Arc<Inner>);

struct Inner {
    client: proxy_client::Client,
    auth: Mutex<Box<dyn Authenticator>>,
    requires_auth: AtomicBool,
}

impl Inner {
    async fn auth_step(
        &self,
        last_headers: Option<hyper::HeaderMap>,
    ) -> crate::auth::Result<hyper::HeaderMap> {
        let auth = self.auth.lock();
        auth.step(last_headers)
    }
}

impl ProxyClient {
    pub fn new(client: proxy_client::Client, auth: Box<dyn Authenticator>) -> Self {
        Self(Arc::new(Inner {
            client,
            auth: Mutex::new(auth),
            requires_auth: AtomicBool::new(true),
        }))
    }

    pub async fn request(&self, mut req: hyper::Request<Body>) -> Result<Response<Body>> {
        if self.0.requires_auth.load(Ordering::Relaxed) {
            let headers = self.0.auth_step(None).await;
            match headers {
                Ok(headers) => req.headers_mut().extend(headers),
                Err(ref cause) => {
                    tracing::error!(?cause, "proxy authentication error");
                    self.0.requires_auth.store(false, Ordering::Relaxed);
                }
            }
        }
        let res = self.0.client.request(req).await?;
        if res.status() == http::StatusCode::PROXY_AUTHENTICATION_REQUIRED {
            let remote_addr = res
                .extensions()
                .get::<HttpProxyInfo>()
                .map(|i| i.remote_addr.to_string())
                .unwrap_or_default();
            tracing::error!(?remote_addr, "proxy requires authentication");
            self.0.requires_auth.store(true, Ordering::Relaxed);
        }
        Ok(res)
    }
}

type ResponseFuture = Pin<Box<dyn Future<Output = Result<Response<Body>>> + Send>>;

pub trait ForwardClient {
    fn connect(&self, req: hyper::Request<Body>) -> ResponseFuture;
    fn http(&self, req: hyper::Request<Body>) -> ResponseFuture;
}

impl ForwardClient for hyper::Client<hyper::client::HttpConnector, Body> {
    fn connect(&self, req: http::Request<Body>) -> ResponseFuture {
        let resp = async move {
            if let Ok(mut stream) = crate::net::dial(req.uri()).await {
                tracing::trace!("connected to: {:?}", stream.peer_addr().ok());
                tokio::task::spawn(async move {
                    match hyper::upgrade::on(req).await {
                        Ok(mut upgraded) => {
                            if let Err(e) = copy_bidirectional(&mut upgraded, &mut stream).await {
                                tracing::error!("tunnel error: {}", e)
                            }
                        }
                        Err(e) => tracing::error!("upgrade error: {}", e),
                    }
                });

                Ok(Response::new(Body::empty()))
            } else {
                tracing::error!("CONNECT host is not socket addr: {:?}", req.uri());
                let mut resp = Response::new(Body::from("CONNECT must be to a socket address"));
                *resp.status_mut() = http::StatusCode::BAD_REQUEST;

                Ok(resp)
            }
        };
        let resp = resp.instrument(tracing::trace_span!("HyperClient::connect"));
        Box::pin(resp)
    }

    fn http(&self, req: http::Request<Body>) -> ResponseFuture {
        let this = self.clone();
        let resp = async move { this.request(req).await.map_err(Error::Hyper) };
        let resp = resp.instrument(tracing::trace_span!("HyperClient::connect"));
        Box::pin(resp)
    }
}

impl ForwardClient for ProxyClient {
    fn connect(&self, mut req: http::Request<Body>) -> ResponseFuture {
        let this = self.clone();

        let resp = async move {
            // Make a client CONNECT request to the parent proxy to upgrade the connection
            let parent_authority = authority_of(req.uri())?;
            let parent_req = Request::connect(parent_authority.clone())
                .version(http::version::Version::HTTP_11)
                .body(Body::empty())
                .unwrap();

            tracing::debug!("forward_connect req: {:?}", req);
            let parent_res = this.request(parent_req).await?;

            if parent_res.status() == StatusCode::OK {
                // Upgrade connection to parent proxy
                match hyper::upgrade::on(parent_res).await {
                    Ok(mut parent_upgraded) => {
                        // On a successful upgrade to the parent proxy, upgrade the
                        // request of the client (the original request maker)
                        tokio::task::spawn(async move {
                            match hyper::upgrade::on(&mut req).await {
                                Ok(mut client_upgraded) => {
                                    if let Err(cause) = copy_bidirectional(
                                        &mut parent_upgraded,
                                        &mut client_upgraded,
                                    )
                                    .await
                                    {
                                        tracing::error!(?cause, "tunnel error")
                                    }
                                }
                                Err(cause) => tracing::error!(?cause, "upgrade error"),
                            }
                        });
                        // Response with a OK to the client
                        Ok(Response::new(Body::empty()))
                    }
                    Err(cause) => Err(Error::UpgradeFailed(cause, parent_authority)),
                }
            } else {
                let status = parent_res.status();
                tracing::error!(?status, "CONNECT {}", &parent_authority);

                Err(Error::ConnectFailed(parent_authority))
            }
        };
        let resp = resp.instrument(tracing::trace_span!("ProxyClient::connect"));
        Box::pin(resp)
    }

    fn http(&self, req: hyper::Request<Body>) -> ResponseFuture {
        let this = self.clone();
        let resp = async move {
            let res = this.request(req).await?;
            Ok(res)
        };
        let resp = resp.instrument(tracing::trace_span!("ProxyClient::http"));
        Box::pin(resp)
    }
}

fn authority_of(uri: &http::Uri) -> Result<http::Uri> {
    match (uri.scheme(), uri.host(), uri.port()) {
        (Some(scheme), Some(host), None) => {
            let port = if *scheme == http::uri::Scheme::HTTP {
                "80"
            } else if *scheme == http::uri::Scheme::HTTPS {
                "443"
            } else {
                return Err(Error::InvalidUri);
            };
            let uri = format!("{}:{}", host, port);
            let uri = uri.parse().map_err(|_| Error::InvalidUri)?;
            Ok(uri)
        }
        (_, Some(host), Some(port)) => {
            let uri = format!("{}:{}", host, port);
            let uri = uri.parse().map_err(|_| Error::InvalidUri)?;
            Ok(uri)
        }
        (_, _, _) => Err(Error::InvalidUri),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn authority_of_test() -> Result<(), Box<dyn std::error::Error>> {
        use super::authority_of;

        // No scheme and no port is an error
        assert!(authority_of(&("example.org".parse()?)).is_err());
        assert_eq!(
            authority_of(&("example.org:8080".parse()?))?
                .port_u16()
                .unwrap(),
            8080
        );
        assert_eq!(
            authority_of(&("http://example.org".parse()?))?
                .port_u16()
                .unwrap(),
            80
        );
        assert_eq!(
            authority_of(&("https://example.org".parse()?))?
                .port_u16()
                .unwrap(),
            443
        );

        Ok(())
    }
}
