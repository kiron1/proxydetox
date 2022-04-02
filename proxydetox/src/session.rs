#![allow(clippy::type_complexity)]

use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::{
    collections::HashMap,
    task::{self, Poll},
};

use futures_util::stream::StreamExt;
use http::header::{ACCEPT, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, HOST, USER_AGENT, VIA};
use http::HeaderValue;
use http::Uri;
use http::{Request, Response};
use hyper::service::Service;
use hyper::Body;
use parking_lot::Mutex;
use proxy_client::HttpProxyConnector;
use tokio::sync::broadcast::{self, Sender};
use tokio_stream::wrappers::BroadcastStream;
use tracing_attributes::instrument;
use tracing_futures::Instrument;

use crate::accesslog;
use crate::auth::AuthenticatorFactory;
use paclib::proxy::ProxyDesc;
use paclib::Evaluator;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("authentication mechanism error: {0}")]
    Auth(#[from] crate::auth::Error),
    #[error("invalid URI")]
    InvalidUri,
    #[error("upstream proxy ({0}) requires authentication")]
    ProxyAuthenticationRequired(ProxyDesc),
}

impl From<crate::client::Error> for Error {
    fn from(cause: crate::client::Error) -> Error {
        use crate::client;
        match cause {
            client::Error::Auth(cause) => Error::Auth(cause),
            client::Error::Hyper(cause) => Error::Hyper(cause),
            client::Error::InvalidUri => Error::InvalidUri,
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

type ProxyClient = crate::client::Client;

#[derive(Debug)]
pub struct Builder {
    pac_script: Option<String>,
    auth: Option<AuthenticatorFactory>,
    pool_max_idle_per_host: usize,
    pool_idle_timeout: Option<Duration>,
    always_use_connect: bool,
}

impl std::default::Default for Builder {
    fn default() -> Self {
        Self {
            pac_script: None,
            auth: None,
            pool_max_idle_per_host: usize::MAX,
            pool_idle_timeout: None,
            always_use_connect: false,
        }
    }
}

impl Builder {
    /// PAC script used for evaluation
    /// If `None`, FindProxy will evaluate to DIRECT
    pub fn pac_script(mut self, pac_script: Option<String>) -> Self {
        self.pac_script = pac_script;
        self
    }
    /// Authenticator factory (Basic or Negotiate)
    /// If `None`, use no authentication toward the proxy.
    pub fn authenticator_factory(mut self, factory: Option<AuthenticatorFactory>) -> Self {
        self.auth = factory;
        self
    }
    /// sets the maximum idle connection per host allowed in the pool
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = max;
        self
    }
    /// set an optional timeout for idle sockets being kept-aliv.
    pub fn pool_idle_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.pool_idle_timeout = timeout;
        self
    }
    /// use the CONNECT method even for HTTP requests.
    pub fn always_use_connect(mut self, yesno: bool) -> Self {
        self.always_use_connect = yesno;
        self
    }

    pub fn build(self) -> Session {
        let pac_script = self
            .pac_script
            .unwrap_or_else(|| "function FindProxyForURL(url, host) { return \"DIRECT\"; }".into());
        let eval = Mutex::new(Evaluator::new(&pac_script).unwrap());
        let auth = self.auth.unwrap_or(AuthenticatorFactory::None);
        let direct_client = hyper::Client::builder()
            .pool_max_idle_per_host(self.pool_max_idle_per_host)
            .pool_idle_timeout(self.pool_idle_timeout)
            .build_http();
        let (accesslog_tx, mut accesslog_rx) = broadcast::channel(16);
        tokio::spawn(async move {
            loop {
                let entry = accesslog_rx.recv().await;
                if let Err(cause) = entry {
                    if cause == broadcast::error::RecvError::Closed {
                        break;
                    }
                }
            }
        });
        Session(Arc::new(Shared {
            eval,
            direct_client,
            proxy_clients: Default::default(),
            auth,
            pool_idle_timeout: self.pool_idle_timeout,
            pool_max_idle_per_host: self.pool_max_idle_per_host,
            always_use_connect: self.always_use_connect,
            accesslog_tx,
        }))
    }
}

#[derive(Clone)]
pub struct Session(Arc<Shared>);

#[derive(Clone)]
pub struct PeerSession {
    peer: Arc<SocketAddr>,
    shared: Arc<Shared>,
}

struct Shared {
    eval: Mutex<paclib::Evaluator>,
    direct_client: hyper::Client<hyper::client::HttpConnector>,
    proxy_clients: Mutex<HashMap<Uri, ProxyClient>>,
    auth: AuthenticatorFactory,
    pool_max_idle_per_host: usize,
    pool_idle_timeout: Option<Duration>,
    always_use_connect: bool,
    accesslog_tx: Sender<accesslog::Entry>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session").finish()
    }
}

impl Session {
    pub fn builder() -> Builder {
        Default::default()
    }
}

impl Shared {
    fn find_proxy(&self, uri: &Uri) -> paclib::Proxies {
        tokio::task::block_in_place(move || {
            self.eval.lock().find_proxy(uri).unwrap_or_else(|cause| {
                tracing::error!("failed to find_proxy: {:?}", cause);
                paclib::Proxies::direct()
            })
        })
    }

    #[instrument(level = "trace", skip(self))]
    fn proxy_client(&self, uri: http::Uri) -> Result<ProxyClient> {
        let mut proxies = self.proxy_clients.lock();
        match proxies.get(&uri) {
            Some(proxy) => Ok(proxy.clone()),
            None => {
                tracing::debug!("new proxy client for {:?}", uri.host());
                let auth = self.auth.make(&uri);
                let auth = match auth {
                    Ok(auth) => auth,
                    Err(ref cause) => {
                        tracing::warn!("error makeing authenticator: {}", cause);
                        Box::new(crate::auth::NoneAuthenticator)
                    }
                };
                let client = hyper::Client::builder()
                    .pool_max_idle_per_host(self.pool_max_idle_per_host)
                    .pool_idle_timeout(self.pool_idle_timeout)
                    .build(HttpProxyConnector::new(uri.clone()));
                let client = ProxyClient::new(client, auth);
                proxies.insert(uri, client.clone());
                Ok(client)
            }
        }
    }
}

impl PeerSession {
    pub async fn process(&mut self, req: hyper::Request<Body>) -> Result<Response<Body>> {
        let res = if req.uri().authority().is_some() {
            self.dispatch(req).await
        } else if req.method() == hyper::Method::GET {
            self.management(req).await
        } else {
            let mut res = Response::new(Body::from(String::from("Invalid Requst")));
            *res.status_mut() = http::StatusCode::BAD_REQUEST;
            Ok(res)
        };

        tracing::debug!("response: {:?}", res.as_ref().map(|r| r.status()));
        res
    }

    pub async fn dispatch(&mut self, mut req: hyper::Request<Body>) -> Result<Response<Body>> {
        // Remove hop-by-hop headers which are meant for the proxy.
        // "proxy-connection" is not an official header, but used by many clients.
        let _proxy_connection = req
            .headers_mut()
            .remove(http::header::HeaderName::from_static("proxy-connection"));
        let _proxy_auth = req.headers_mut().remove(http::header::PROXY_AUTHORIZATION);

        let proxy = self.shared.find_proxy(req.uri()).first();
        self.dispatch_with_proxy(proxy, req).await
    }

    #[instrument(level = "debug", skip(self, req))]
    pub async fn dispatch_with_proxy(
        &mut self,
        proxy: ProxyDesc,
        req: hyper::Request<Body>,
    ) -> Result<Response<Body>> {
        let proxy_client;
        let client: &(dyn crate::client::ForwardClient + Send + Sync) = match proxy {
            ProxyDesc::Direct => &self.shared.direct_client,
            ProxyDesc::Proxy(ref proxy) => {
                proxy_client = self.shared.proxy_client(proxy.clone())?;
                &proxy_client
            }
        };

        let access = accesslog::Entry::begin(
            *self.peer,
            proxy.clone(),
            req.method().clone(),
            req.uri().clone(),
            req.version(),
            req.headers()
                .get(USER_AGENT)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_owned()),
        );

        let is_connect = req.method() == hyper::Method::CONNECT || self.shared.always_use_connect;
        let mut res = if is_connect {
            client.connect(req).await
        } else {
            client.http(req).await
        };

        match res {
            Ok(ref mut res) => {
                let entry = access.success(
                    res.status(),
                    res.headers()
                        .get(CONTENT_LENGTH)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok()),
                );
                let _ = self.shared.accesslog_tx.send(entry);

                if res.status() == http::StatusCode::PROXY_AUTHENTICATION_REQUIRED {
                    tracing::error!("407 proxy authentication required for {}", &proxy);
                    return Err(Error::ProxyAuthenticationRequired(proxy));
                }

                let via = HeaderValue::from_str(&format!(
                    "{}; {}/{}",
                    &proxy,
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                ))
                .unwrap();
                res.headers_mut().insert(VIA, via);
            }
            Err(ref cause) => {
                let entry = access.error(&cause);
                let _ = self.shared.accesslog_tx.send(entry);
            }
        }
        Ok(res?)
    }

    pub async fn management(&mut self, req: hyper::Request<Body>) -> Result<Response<Body>> {
        let resp = if req.uri() == "/access.log" {
            let accept_event_stream = req
                .headers()
                .get(ACCEPT)
                .map(|v| v == "text/event-stream")
                .unwrap_or(false);

            if accept_event_stream {
                // the client accepts an SSE event stream
                let stream = self.shared.accesslog_tx.subscribe();
                let stream = BroadcastStream::new(stream);
                let stream = stream.map(|entry| match entry {
                    Ok(entry) => {
                        let chunk = format!("data:{}\n\n", entry);
                        std::result::Result::<_, std::io::Error>::Ok(chunk)
                    }
                    Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(
                        count,
                    )) => {
                        let chunk = format!("event:lagged\ndata:{}\n\n", count);
                        Ok(chunk)
                    }
                });

                let mut resp = Response::new(Body::wrap_stream(stream));
                resp.headers_mut().insert(
                    CACHE_CONTROL,
                    http::header::HeaderValue::from_static("no-store"),
                );
                resp.headers_mut().insert(
                    CONTENT_TYPE,
                    http::header::HeaderValue::from_static("text/event-stream"),
                );
                resp
            } else {
                let body = include_str!("accesslog.html");
                let mut resp = Response::new(Body::from(body));
                resp.headers_mut().insert(
                    CONTENT_TYPE,
                    http::header::HeaderValue::from_static("text/html"),
                );
                resp
            }
        } else if req.uri() == "/proxy.pac" {
            let body = format!(
                "function FindProxyForURL(url, host) {{ return \"PROXY {}\"; }}\n",
                req.headers()
                    .get(HOST)
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("127.0.0.1:3128")
            );
            let mut resp = Response::new(Body::from(body));
            resp.headers_mut().insert(
                CONTENT_TYPE,
                http::header::HeaderValue::from_static("application/x-ns-proxy-autoconfig"),
            );
            resp
        } else {
            let body = format!(
                "<!DOCTYPE html><html><h1>{}/{}</h1><h2>DNS Cache</h2><code>{:?}</code></html>",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                self.shared.eval.lock().cache()
            );
            let mut resp = Response::new(Body::from(body));
            resp.headers_mut().insert(
                CONTENT_TYPE,
                http::header::HeaderValue::from_static("text/html"),
            );
            resp
        };
        Ok(resp)
    }
}

fn make_error_response<E>(error: &E) -> Response<Body>
where
    E: std::error::Error + Send + Sync,
{
    let body = format!(
        include_str!("502.html"),
        error,
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

impl Service<Request<Body>> for PeerSession {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut task::Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut this = self.clone();
        let method = req.method().clone();
        let uri = req.uri().clone();
        let version = req.version();

        let res = async move {
            tracing::trace!("request: {:?}", &req);
            let res = this.process(req).await;
            tracing::trace!("response: {:?}", &res);
            let out = match res {
                Err(ref error) => make_error_response(error),
                Ok(res) => res,
            };
            Ok(out)
        };
        let res = res
            .instrument(tracing::debug_span!("call", method=%method, uri=%uri, version=?version));
        Box::pin(res)
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
        let res = res.instrument(tracing::debug_span!("call", addr=%addr));
        Box::pin(res)
    }
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
