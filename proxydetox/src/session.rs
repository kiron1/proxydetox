#![allow(clippy::type_complexity)]

use std::convert::Infallible;
use std::fmt::Write;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::{
    collections::HashMap,
    task::{self, Poll},
};

use futures_util::stream::StreamExt;
use http::header::{ACCEPT, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, HOST, USER_AGENT};
use http::Uri;
use http::{Request, Response};
use hyper::Body;
use parking_lot::Mutex;
use proxy_client::HttpProxyConnector;
use tokio::sync::broadcast::{self, Sender};
use tokio_stream::wrappers::BroadcastStream;
use tower::{util::BoxService, Service, ServiceExt};
use tracing_attributes::instrument;
use tracing_futures::Instrument;

use crate::accesslog;
use crate::auth::AuthenticatorFactory;
use crate::client::ProxyClient;
use crate::connect::Connect;
use detox_net::HostAndPort;
use paclib::proxy::ProxyDesc;
use paclib::Evaluator;

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

    #[error("upstream error reaching {2} via {1}: {0}")]
    Upstream(#[source] crate::client::Error, HostAndPort, Uri),
    #[error("error creating client for {1}: {0}")]
    MakeClient(#[source] hyper::Error, Uri),
    #[error("error creating proxy for {1}: {0}")]
    MakeProxyClient(#[source] crate::client::Error, HostAndPort),
    #[error("client error: {0}")]
    Client(
        #[from]
        #[source]
        hyper::Error,
    ),
    #[error("connect error reaching {1}: {0}")]
    Connect(#[source] tokio::io::Error, Uri),
    #[error("proxy connect error reaching {2} via {1}: {0}")]
    ProxyConnect(#[source] crate::client::ConnectError, HostAndPort, Uri),
    #[error("upstream proxy ({0}) requires authentication")]
    ProxyAuthenticationRequired(ProxyDesc),
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

#[derive(Debug, Default)]
pub struct Builder {
    pac_script: Option<String>,
    auth: Option<AuthenticatorFactory>,
    always_use_connect: bool,
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
    /// use the CONNECT method even for HTTP requests.
    pub fn always_use_connect(mut self, yesno: bool) -> Self {
        self.always_use_connect = yesno;
        self
    }

    pub fn build(self) -> Session {
        let pac_script = self
            .pac_script
            .unwrap_or_else(|| crate::DEFAULT_PAC_SCRIPT.into());
        let eval = Mutex::new(Evaluator::new(&pac_script).unwrap());
        let auth = self.auth.unwrap_or(AuthenticatorFactory::None);
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
            direct_client: Mutex::new(Default::default()),
            proxy_clients: Default::default(),
            auth,
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
    direct_client: Mutex<crate::client::Direct>,
    proxy_clients: Mutex<HashMap<HostAndPort, ProxyClient>>,
    auth: AuthenticatorFactory,
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
                tracing::error!(%cause, %uri, "failed to find_proxy");
                paclib::Proxies::direct()
            })
        })
    }

    fn proxy_for(&self, endpoint: HostAndPort) -> Result<ProxyClient> {
        let mut proxies = self.proxy_clients.lock();
        match proxies.get(&endpoint) {
            Some(proxy) => Ok(proxy.clone()),
            None => {
                tracing::debug!(endpoint=%endpoint, "new proxy client");
                let auth = self.auth.make(endpoint.host());
                let auth = match auth {
                    Ok(auth) => auth,
                    Err(ref cause) => {
                        tracing::warn!(%cause, "error makeing authenticator");
                        Box::new(crate::auth::NoneAuthenticator)
                    }
                };
                let client =
                    hyper::Client::builder().build(HttpProxyConnector::new(endpoint.clone()));
                let client = ProxyClient::new(client, auth);
                proxies.insert(endpoint, client.clone());
                Ok(client)
            }
        }
    }

    #[instrument(level = "trace", skip(self))]
    fn proxy_client(
        &self,
        proxy: HostAndPort,
    ) -> Result<BoxService<Request<Body>, Response<Body>, Error>> {
        let client = self.proxy_for(proxy.clone());
        client.map(|s| s.map_err(move |e| Error::MakeProxyClient(e, proxy)).boxed())
    }

    async fn proxy_connect(
        &self,
        proxy: HostAndPort,
        uri: http::Uri,
    ) -> Result<BoxService<Request<Body>, Response<Body>, Error>> {
        let proxy_client = self.proxy_for(proxy.clone())?;
        let host = HostAndPort::try_from_uri(&uri)?;
        proxy_client
            .connect(host)
            .await
            .map_err({
                let proxy = proxy.clone();
                let uri = uri.clone();
                move |e| Error::ProxyConnect(e, proxy, uri)
            })
            .map(move |c| c.map_err(|e| Error::Upstream(e, proxy, uri)).boxed())
    }

    async fn direct_client(
        &self,
        uri: http::Uri,
    ) -> Result<BoxService<Request<Body>, Response<Body>, Error>> {
        let client = {
            let uri = uri.clone();
            let mut guard = self.direct_client.lock();
            guard.call(uri)
        };
        client
            .await
            .map_err(move |e| Error::MakeClient(e, uri))
            .map(|s| s.map_err(Error::Client).boxed())
    }

    /// Establish a connection to parent proxy.
    ///
    /// In case of `CONNECT` the connesction will be established so far that `CONNECT` request is
    /// send, but not the client request.
    /// For upstream servers which can be connected directly a TCP connection will be established.
    /// For a directly reachable server with a regular HTTP request, no action will be perforemd.
    #[instrument(skip(self, method, uri))]
    async fn establish_connection(
        &self,
        proxy: paclib::ProxyDesc,
        method: &http::Method,
        uri: &http::Uri,
    ) -> Result<BoxService<Request<Body>, Response<Body>, Error>> {
        let is_connect = method == hyper::Method::CONNECT;
        let use_connect = self.always_use_connect;

        match (is_connect, use_connect, proxy) {
            (true, _, ProxyDesc::Proxy(proxy)) => self.proxy_connect(proxy, uri.clone()).await,
            (false, true, ProxyDesc::Proxy(proxy)) => self.proxy_connect(proxy, uri.clone()).await,
            (false, false, ProxyDesc::Proxy(proxy)) => self.proxy_client(proxy),
            (true, _, ProxyDesc::Direct) => {
                let mut conn = Connect::new();
                let handshake = conn.call(uri.clone()).await;
                handshake
                    .map_err({
                        let uri = uri.clone();
                        move |e| Error::Connect(e, uri)
                    })
                    .map(|s| s.map_err(|_| Error::Handshake).boxed())
            }
            (false, _, ProxyDesc::Direct) => self.direct_client(uri.clone()).await,
        }
    }
}

impl PeerSession {
    async fn process(&mut self, req: hyper::Request<Body>) -> Result<Response<Body>> {
        let res = if req.uri().authority().is_some() {
            self.dispatch(req).await
        } else if req.method() != hyper::Method::CONNECT {
            self.management(req).await
        } else {
            make_error_html(http::StatusCode::BAD_REQUEST, "Invalid request")
        };

        tracing::debug!(status=?res.as_ref().map(|r| r.status()), "response");
        res
    }

    async fn dispatch(&mut self, mut req: hyper::Request<Body>) -> Result<Response<Body>> {
        let access = accesslog::Entry::begin(
            *self.peer,
            req.method().clone(),
            req.uri().clone(),
            req.version(),
            req.headers()
                .get(USER_AGENT)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_owned()),
        );
        // Remove hop-by-hop headers which are meant for the proxy.
        // "proxy-connection" is not an official header, but used by many clients.
        let _proxy_connection = req
            .headers_mut()
            .remove(http::header::HeaderName::from_static("proxy-connection"));
        let _proxy_auth = req.headers_mut().remove(http::header::PROXY_AUTHORIZATION);

        let proxies = {
            let mut proxies = self.shared.find_proxy(req.uri());
            // If the returned list of proxies does not contain a `DIRECT`, add one as fall back optoin
            // in case all connections attempts fail.
            if !proxies.iter().any(|p| *p == ProxyDesc::Direct) {
                proxies.push(ProxyDesc::Direct);
            }
            proxies
        };

        let conn = self
            .establish_connection(proxies, req.method(), req.uri())
            .await;

        let (res, proxy) = match conn {
            Ok((mut client, proxy)) => (client.call(req).await, proxy),
            Err(e) => return Err(e),
        };

        let entry = match &res {
            Ok(res) => access.success(
                proxy.clone(),
                res.status(),
                res.headers()
                    .get(CONTENT_LENGTH)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok()),
            ),
            Err(cause) => {
                tracing::error!(%cause, "HTTP upstream error");
                access.error(proxy.clone(), cause)
            }
        };
        let _ = self.shared.accesslog_tx.send(entry);

        res.and_then(|res| {
            if res.status() == http::StatusCode::PROXY_AUTHENTICATION_REQUIRED {
                tracing::error!(%proxy, "407 proxy authentication required");
                Err(Error::ProxyAuthenticationRequired(proxy))
            } else {
                Ok(res)
            }
        })
    }

    #[instrument(skip(self, method, uri))]
    async fn establish_connection(
        &mut self,
        proxies: paclib::Proxies,
        method: &http::Method,
        uri: &http::Uri,
    ) -> Result<(BoxService<Request<Body>, Response<Body>, Error>, ProxyDesc)> {
        let mut client: Result<BoxService<Request<Body>, Response<Body>, Error>> =
            Err(Error::UnableToEstablishConnection(uri.clone()));
        let mut upstream_proxy: Option<paclib::ProxyDesc> = None;

        for proxy in proxies.iter() {
            tracing::debug!(%proxy, "try connect");
            let conn = self
                .shared
                .establish_connection(proxy.to_owned(), method, uri)
                .await;
            match conn {
                Ok(conn) => {
                    tracing::debug!(%proxy, "connection established");
                    client = Ok(conn);
                    upstream_proxy = Some(proxy.to_owned());
                    break;
                }
                Err(cause) => tracing::warn!(%cause, "connecting failed"),
            }
        }

        if client.is_err() {
            tracing::error!("unable to establish connection");
        }

        let proxy = upstream_proxy.unwrap_or(ProxyDesc::Direct);
        client.map(move |c| (c, proxy))
    }

    async fn management(&mut self, req: hyper::Request<Body>) -> Result<Response<Body>> {
        const GET: http::Method = http::Method::GET;
        let accept = req.headers().get(ACCEPT).and_then(|s| s.to_str().ok());

        match (req.method(), accept, req.uri().path()) {
            (&GET, _, "/") => self.index_html(),
            (&GET, Some("text/event-stream"), "/access.log") => self.accesslog_stream(),
            (&GET, _, "/access.log") => self.accesslog_html(),
            (&GET, _, "/proxy.pac") => proxy_pac(req.headers().get(HOST)),
            (&GET, _, _) => make_error_html(http::StatusCode::NOT_FOUND, "ressource not found"),
            (_, _, _) => {
                make_error_html(http::StatusCode::METHOD_NOT_ALLOWED, "method not allowed")
            }
        }
    }

    fn index_html(&self) -> Result<Response<Body>> {
        let version = if let Some(hash) = option_env!("PROXYDETOX_BUILD_GIT_HASH") {
            format!("{} ({})", env!("CARGO_PKG_VERSION"), hash)
        } else {
            env!("CARGO_PKG_VERSION").to_owned()
        };
        let body = format!(
            "<!DOCTYPE html><html><h1>{}/{}</h1><h2>DNS Cache</h2><code>{:?}</code></html>",
            env!("CARGO_PKG_NAME"),
            version,
            self.shared.eval.lock().cache()
        );
        let resp = Response::builder()
            .header(
                CONTENT_TYPE,
                http::header::HeaderValue::from_static("text/html"),
            )
            .body(Body::from(body))?;
        Ok(resp)
    }

    fn accesslog_html(&self) -> Result<Response<Body>> {
        let resp = Response::builder()
            .header(
                CONTENT_TYPE,
                http::header::HeaderValue::from_static("text/html"),
            )
            .body(Body::from(include_str!("accesslog.html")))?;
        Ok(resp)
    }

    fn accesslog_stream(&self) -> Result<Response<Body>> {
        // the client accepts an SSE event stream
        let stream = self.shared.accesslog_tx.subscribe();
        let stream = BroadcastStream::new(stream);
        let stream = stream.map(|entry| match entry {
            Ok(entry) => {
                let chunk = format!("data:{}\n\n", entry);
                std::result::Result::<_, std::io::Error>::Ok(chunk)
            }
            Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(count)) => {
                let chunk = format!("event:lagged\ndata:{}\n\n", count);
                Ok(chunk)
            }
        });

        let resp = Response::builder()
            .header(
                CACHE_CONTROL,
                http::header::HeaderValue::from_static("no-store"),
            )
            .header(
                CONTENT_TYPE,
                http::header::HeaderValue::from_static("text/event-stream"),
            )
            .body(Body::wrap_stream(stream))?;
        Ok(resp)
    }
}

fn proxy_pac(host: Option<&http::HeaderValue>) -> Result<Response<Body>> {
    let body = format!(
        "function FindProxyForURL(url, host) {{ return \"PROXY {}\"; }}\n",
        host.and_then(|h| h.to_str().ok())
            .unwrap_or("127.0.0.1:3128")
    );
    let resp = Response::builder()
        .header(
            CONTENT_TYPE,
            http::header::HeaderValue::from_static("application/x-ns-proxy-autoconfig"),
        )
        .body(Body::from(body))?;
    Ok(resp)
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
    write!(&mut description, "<p><strong>Error:</strong> {}</p>", error).ok();
    if let Some(cause) = error.source() {
        description
            .write_str("<p><strong>Caused by:</strong></p><ol reversed>")
            .ok();
        for msg in std::iter::successors(Some(cause), |e| e.source()) {
            write!(&mut description, "<li>{}</li>", msg).ok();
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
            tracing::trace!(?req, "request");
            let res = this.process(req).await;
            tracing::trace!(?res, "response");
            let out = match res {
                Err(ref error) => make_error_response(error),
                Ok(res) => res,
            };
            Ok(out)
        };
        let res =
            res.instrument(tracing::info_span!("call", method=%method, uri=%uri, version=?version));
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
