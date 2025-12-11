use crate::context::Context;
use crate::{accesslog, body};
use bytes::Bytes;
use detox_net::{HostAndPort, copy_bidirectional};
use futures_util::{FutureExt, StreamExt, stream};
use http::Uri;
use http::header::{
    CACHE_CONTROL, CONNECTION, CONTENT_LENGTH, CONTENT_TYPE, HOST, PROXY_AUTHENTICATE,
    PROXY_AUTHORIZATION, TE, TRAILER, TRANSFER_ENCODING, UPGRADE, USER_AGENT,
};
use http::{HeaderMap, HeaderName};
use http::{HeaderValue, Response};
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use hyper_util::rt::TokioIo;
use lazy_static::lazy_static;
use paclib::proxy::Proxy;
use paclib::{Proxies, ProxyOrDirect};
use std::convert::Infallible;
use std::fmt::Write;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tracing_attributes::instrument;

type Body = BoxBody<Bytes, hyper::Error>;

lazy_static! {
    // "proxy-connection" is not an official header, but used by many clients.
    // https://stackoverflow.com/a/62722840
    static ref PROXY_CONNECTION: HeaderName = HeaderName::from_static("proxy-connection");
    static ref HOP_BY_HOP_HEADERS: [&'static HeaderName; 7] = [
        // &CONNECTION is already handled
        &PROXY_AUTHENTICATE,
        &PROXY_AUTHORIZATION,
        &PROXY_CONNECTION,
        &TE,
        &TRAILER,
        &TRANSFER_ENCODING,
        // A server MAY ignore a received Upgrade header field if it wishes to
        // continue using the current protocol on that connection. Upgrade
        // cannot be used to insist on a protocol change. [...] A server MUST
        // NOT switch to a protocol that was not indicated by the client in the
        // corresponding request's Upgrade header field.
        // https://datatracker.ietf.org/doc/html/rfc7230#section-6.7

        // _Proxy Usage_: If the client is configured to use a proxy when
        // using the WebSocket Protocol to connect to host /host/ and port
        // /port/, then the client SHOULD connect to that proxy and ask it
        // to open a TCP connection to the host given by /host/ and the port
        // given by /port/.
        // https://datatracker.ietf.org/doc/html/rfc6455#section-4.1
        &UPGRADE,
    ];
}

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
    ConnectTimeout(Proxy, Box<Uri>),
    #[error("error when connecting to {1} via proxy {0}")]
    ConnectionFailed(Proxies, Box<Uri>),
    #[error("connetion error: {0}")]
    Connection(
        #[from]
        #[source]
        detox_hyper::conn::Error,
    ),
    #[error("client error: {0}")]
    Client(
        #[from]
        #[source]
        hyper::Error,
    ),
    #[error("connect error reaching {1}: {0}")]
    Connect(#[source] tokio::io::Error, Box<Uri>),
    #[error("upstream proxy ({0}) requires authentication")]
    ProxyAuthenticationRequired(HostAndPort),
    #[error("received invalid status code: {0}")]
    InvalidStatusCode(http::StatusCode),
    #[error("http error: {0}")]
    Http(
        #[source]
        #[from]
        http::Error,
    ),
    #[error("unable to establish connection: {0}")]
    UnableToEstablishConnection(Box<Uri>),
    #[error("handshake error")]
    Handshake,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Session(Arc<Inner>);

struct Inner {
    context: Arc<Context>,
    addr: SocketAddr,
}

impl Session {
    pub fn new(context: Arc<Context>, addr: SocketAddr) -> Self {
        Self(Arc::new(Inner { context, addr }))
    }
}

impl Inner {
    #[instrument(level = "debug", skip(self, req), fields(addr = debug(self.addr), http.method = debug(req.method()), http.uri = debug(req.uri())))]
    async fn handle(
        &self,
        req: http::Request<hyper::body::Incoming>,
    ) -> std::result::Result<http::Response<BoxBody<Bytes, hyper::Error>>, Infallible> {
        // TODO: management console must also be choosen, when authority is pointing to us
        // (or abort the connection), since otherwise we create an endless loop.
        let res = if req.uri().authority().is_some() {
            self.proxy_request(req).await
        } else if req.method() != hyper::Method::CONNECT {
            self.management_console(req).await
        } else {
            Ok(make_error_html(
                http::StatusCode::BAD_REQUEST,
                format!("Invalid request: <tt>{} {}</<tt>", req.method(), req.uri()),
            ))
        };
        match res {
            Ok(res) => Ok::<_, Infallible>(res),
            Err(cause) => Ok::<_, Infallible>(make_error_response(&cause)),
        }
    }

    async fn proxy_request(
        &self,
        mut req: http::Request<hyper::body::Incoming>,
    ) -> Result<http::Response<BoxBody<Bytes, hyper::Error>>> {
        let access = accesslog::Entry::begin(
            self.addr,
            req.method().clone(),
            req.uri().clone(),
            req.version(),
            req.headers()
                .get(USER_AGENT)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_owned()),
        );
        remove_hop_by_hop_headers(req.headers_mut());
        let uri = if req.uri().scheme().is_some() {
            req.uri().clone()
        } else {
            Uri::builder()
                .scheme(http::uri::Scheme::HTTP)
                .authority(req.uri().authority().cloned().expect("URI with authority"))
                .path_and_query(
                    req.uri()
                        .path_and_query()
                        .cloned()
                        .unwrap_or(http::uri::PathAndQuery::from_static("/")),
                )
                .build()
                .expect("URI")
        };
        let proxies = self.context.find_proxy(uri).await;
        let conn = proxies.clone().into_iter().map({
            let cx = self.context.clone();
            let method = req.method();
            let uri = req.uri();
            move |p| {
                let cx = cx.clone();
                let race = cx.race_connect;
                async move {
                    let r = cx.connect(p, method.clone(), uri.clone()).await;
                    if let Err(ref cause) = r {
                        if race {
                            tracing::debug!(%cause, "unable to connect");
                        } else {
                            tracing::warn!(%cause, "unable to connect");
                        }
                    }
                    r
                }
            }
        });

        let conn = Box::pin(stream::iter(conn));

        let conn = if self.context.race_connect {
            let conn = conn
                .buffer_unordered(self.context.parallel_connect)
                .filter_map(|c| async move { c.ok() });
            let mut conn = Box::pin(conn);
            conn.next().await
        } else {
            let conn = conn
                .buffered(self.context.parallel_connect)
                .filter_map(|c| async move { c.ok() });
            let mut conn = Box::pin(conn);
            conn.next().await
        };
        let proxy = conn
            .as_ref()
            .map(|c| c.proxy().to_owned())
            .unwrap_or(ProxyOrDirect::Direct);

        let resp = if let Some(mut conn) = conn {
            let resp = if req.method() == hyper::Method::CONNECT {
                tokio::task::spawn(async move {
                    match hyper::upgrade::on(req).await {
                        Ok(upgraded) => {
                            let mut upgraded = TokioIo::new(upgraded);
                            if let Err(cause) = copy_bidirectional(&mut upgraded, &mut conn).await {
                                tracing::error!(%cause, "copy bidrectional");
                            }
                        }
                        Err(cause) => tracing::error!(%cause, "upgrade error"),
                    }
                });

                Ok(Response::new(body::empty()))
            } else {
                let conn = conn.handshake().await?;
                let mut resp = conn.send_request(req).await?;
                remove_hop_by_hop_headers(resp.headers_mut());
                Ok(resp.map(|b| b.boxed()))
            };

            match resp {
                Ok(resp) => {
                    match (&proxy, resp.status()) {
                        (
                            ProxyOrDirect::Proxy(proxy),
                            http::StatusCode::PROXY_AUTHENTICATION_REQUIRED,
                        ) => {
                            tracing::error!(%proxy, "407 proxy authentication required");
                            Err(Error::ProxyAuthenticationRequired(
                                proxy.endpoint().to_owned(),
                            ))
                        }
                        (
                            ProxyOrDirect::Direct,
                            http::StatusCode::PROXY_AUTHENTICATION_REQUIRED,
                        ) => {
                            // illegal case, we should never get this response from a non-proxy connection.
                            tracing::error!(status = %resp.status(), "invalid status code from direct connection");
                            Err(Error::InvalidStatusCode(resp.status()))
                        }
                        _ => Ok(resp),
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            Err(Error::ConnectionFailed(
                proxies,
                Box::new(req.uri().clone()),
            ))
        };

        let entry = {
            match &resp {
                Ok(res) => access.success(
                    proxy,
                    res.status(),
                    res.headers()
                        .get(CONTENT_LENGTH)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok()),
                ),
                Err(cause) => access.error(Some(proxy.clone()), cause),
            }
        };
        self.context.accesslog_tx.send(entry).ok();

        resp
    }

    async fn management_console(
        &self,
        req: http::Request<hyper::body::Incoming>,
    ) -> Result<http::Response<Body>> {
        const GET: http::Method = http::Method::GET;
        match (req.method(), req.uri().path()) {
            (&GET, "/") => self.index_html(),
            (&GET, "/access.log") => self.accesslog_stream(),
            (&GET, "/access.html") => self.accesslog_html(),
            (&GET, "/proxy.pac") => proxy_pac(req.headers().get(HOST)),
            (&GET, _) => Ok(make_error_html(
                http::StatusCode::NOT_FOUND,
                "ressource not found",
            )),
            (_, _) => Ok(make_error_html(
                http::StatusCode::METHOD_NOT_ALLOWED,
                "method not allowed",
            )),
        }
    }

    fn index_html(&self) -> Result<Response<Body>> {
        let body = format!(
            "<!DOCTYPE html><html><body><h1>Proxydetox<h1><p>{}/{}</p></body></html>",
            env!("CARGO_PKG_NAME"),
            *crate::VERSION_STR,
        );
        let resp = Response::builder()
            .header(CONTENT_TYPE, HeaderValue::from_static("text/html"))
            .body(body::full(body))?;
        Ok(resp)
    }

    fn accesslog_html(&self) -> Result<Response<Body>> {
        let resp = Response::builder()
            .header(CONTENT_TYPE, HeaderValue::from_static("text/html"))
            .body(body::full(include_str!("accesslog.html")))?;
        Ok(resp)
    }

    fn accesslog_stream(&self) -> Result<Response<Body>> {
        // the client accepts an SSE event stream
        let stream = self.context.accesslog_tx.subscribe();
        let stream = BroadcastStream::new(stream);
        let stream = stream.map(|entry| match entry {
            Ok(entry) => {
                let frame = Frame::data(Bytes::from(format!("data:{entry}\n\n")));
                std::result::Result::<_, hyper::Error>::Ok(frame)
            }
            Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(count)) => {
                let frame = Frame::data(Bytes::from(format!("event:lagged\ndata:{count}\n\n")));
                Ok(frame)
            }
        });

        let resp = Response::builder()
            .header(CACHE_CONTROL, HeaderValue::from_static("no-store"))
            .header(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"))
            .body(BoxBody::new(StreamBody::new(stream)))?;
        Ok(resp)
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session").finish()
    }
}

impl hyper::service::Service<http::Request<hyper::body::Incoming>> for Session {
    type Response = http::Response<BoxBody<Bytes, hyper::Error>>;
    type Error = std::convert::Infallible;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: http::Request<hyper::body::Incoming>) -> Self::Future {
        let this = self.0.clone();
        async move { this.handle(req).await }.boxed()
    }
}

fn make_error_html(
    status: http::StatusCode,
    message: impl AsRef<str>,
) -> Response<BoxBody<Bytes, hyper::Error>> {
    let body = format!(
        include_str!("error.html"),
        status = status,
        message = message.as_ref(),
        name = env!("CARGO_PKG_NAME"),
        version = *crate::VERSION_STR,
    );

    Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, "text/html")
        .header(http::header::CONNECTION, "close")
        .body(crate::body::full(body))
        .expect("error response")
}

fn make_error_response<E>(error: &E) -> Response<BoxBody<Bytes, hyper::Error>>
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
    make_error_html(http::StatusCode::BAD_GATEWAY, description)
}

fn proxy_pac(host: Option<&HeaderValue>) -> std::result::Result<Response<Body>, Error> {
    let body = format!(
        "function FindProxyForURL(url, host) {{ return \"PROXY {}\"; }}\n",
        host.and_then(|h| h.to_str().ok())
            .unwrap_or("127.0.0.1:3128")
    );
    let resp = Response::builder()
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-ns-proxy-autoconfig"),
        )
        .body(body::full(body))?;
    Ok(resp)
}

fn remove_hop_by_hop_headers(headers: &mut HeaderMap) {
    // Remove hop-by-hop headers which must not be forwarded.
    if let Some(connection) = headers.remove(CONNECTION)
        && let Ok(connection) = connection.to_str()
    {
        let iter = connection
            .split(',')
            .map(|h| h.trim())
            .filter(|h| !h.is_empty());
        for name in iter {
            headers.remove(name.trim());
        }
    }
    for header in HOP_BY_HOP_HEADERS.iter() {
        headers.remove(*header);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response() {
        let resp = make_error_response(&super::Error::InvalidUri);
        assert_ne!(resp.status(), http::StatusCode::OK);
    }

    #[test]
    fn remove_hop_by_hop_headers_1() {
        let keep_alive: HeaderName = HeaderName::from_static("keep-alive");
        let mut headers = HeaderMap::new();
        headers.insert(&HOST, HeaderValue::from_static("example.org"));
        headers.insert(&*PROXY_CONNECTION, HeaderValue::from_static("Close"));
        headers.insert(&CONNECTION, HeaderValue::from_static("Keep-Alive"));
        headers.insert(&keep_alive, HeaderValue::from_static("max=1"));
        remove_hop_by_hop_headers(&mut headers);
        assert!(headers.contains_key(&HOST));
        assert!(!headers.contains_key(&*PROXY_CONNECTION));
        assert!(!headers.contains_key(&CONNECTION));
        assert!(!headers.contains_key(&keep_alive));
    }
}
