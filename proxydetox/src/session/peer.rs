use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{self, Poll};

use futures_util::stream::StreamExt;
use http::header::{ACCEPT, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, HOST, USER_AGENT};
use http::{Request, Response};
use hyper::Body;
use tokio_stream::wrappers::BroadcastStream;
use tower::{util::BoxService, Service};
use tracing_attributes::instrument;
use tracing_futures::Instrument;

use super::make_error_html;
use super::make_error_response;
use super::Error;
use super::Result;
use super::Shared;
use crate::accesslog;
use paclib::proxy::ProxyDesc;

#[derive(Clone)]
pub struct PeerSession {
    pub(super) peer: Arc<SocketAddr>,
    pub(super) shared: Arc<Shared>,
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
            if self.shared.direct_fallback {
                // If the returned list of proxies does not contain a `DIRECT`, add one as fall back
                // option in case all connections attempts fail.
                if !proxies.iter().any(|p| *p == ProxyDesc::Direct) {
                    proxies.push(ProxyDesc::Direct);
                }
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
            .body(Body::from(include_str!("../accesslog.html")))?;
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
            res.instrument(tracing::info_span!("call", method=%method, uri=%uri, version=?version, client_addr=%self.peer));
        Box::pin(res)
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
