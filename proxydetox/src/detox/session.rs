use std::convert::Infallible;
use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;
use std::{
    collections::HashMap,
    task::{self, Poll},
};

use futures_util::future::try_join;
use http::header::HOST;
use http::HeaderValue;
use http::Uri;
use http::{Request, Response, StatusCode};
use hyper::service::Service;
use hyper::Body;
use tokio::io::{AsyncRead, AsyncWrite, Error, ErrorKind};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::client::HttpProxyConnector;
use paclib::proxy::ProxyDesc;
use paclib::Evaluator;

#[derive(Debug)]
pub enum SessionError {
    Io(std::io::Error),
    Hyper(hyper::error::Error),
}

impl std::error::Error for SessionError {}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match *self {
            SessionError::Io(ref err) => write!(f, "I/O error: {}", err),
            SessionError::Hyper(ref err) => write!(f, "hyper error: {}", err),
        }
    }
}

impl From<hyper::error::Error> for SessionError {
    fn from(cause: hyper::error::Error) -> SessionError {
        SessionError::Hyper(cause)
    }
}

impl From<std::io::Error> for SessionError {
    fn from(cause: std::io::Error) -> SessionError {
        SessionError::Io(cause)
    }
}

type ProxyClient = hyper::Client<crate::client::HttpProxyConnector>;

#[derive(Clone)]
pub struct DetoxSession {
    eval: Arc<Mutex<paclib::Evaluator>>,
    direct_client: hyper::Client<hyper::client::HttpConnector>,
    proxy_clients: Arc<Mutex<HashMap<Uri, ProxyClient>>>,
}

impl DetoxSession {
    pub fn new(pac_script: &str) -> Self {
        let eval = Arc::new(Mutex::new(Evaluator::new(pac_script).unwrap()));
        let direct_client = Default::default();
        let proxy_clients = Default::default();
        Self {
            eval,
            direct_client,
            proxy_clients,
        }
    }

    // For now just support one single proxy
    async fn find_proxy(&mut self, uri: &http::Uri) -> paclib::proxy::ProxyDesc {
        log::debug!("find proxy for {:?}", &uri);
        self.eval.lock().await.find_proxy(&uri).unwrap().first()
    }

    async fn proxy_client(&mut self, uri: http::Uri) -> ProxyClient {
        let mut proxies = self.proxy_clients.lock().await;
        match proxies.get(&uri) {
            Some(proxy) => proxy.clone(),
            None => {
                let client = hyper::Client::builder().build(HttpProxyConnector::new(uri.clone()));
                proxies.insert(uri, client.clone());
                client
            }
        }
    }

    async fn direct_connect(&mut self, req: Request<Body>) -> Result<Response<Body>, SessionError> {
        // Received an HTTP request like:
        // ```
        // CONNECT www.domain.com:443 HTTP/1.1
        // Host: www.domain.com:443
        // Proxy-Connection: Keep-Alive
        // ```
        //
        // When HTTP method is CONNECT we should return an empty body
        // then we can eventually upgrade the connection and talk a new protocol.
        //
        // Note: only after client received an empty body with STATUS_OK can the
        // connection be upgraded, so we can't return a response inside
        // `on_upgrade` future.
        if let Ok(stream) = dial(req.uri()).await {
            tokio::task::spawn(async move {
                match req.into_body().on_upgrade().await {
                    Ok(upgraded) => match tunnel(upgraded, stream).await {
                        Err(e) => log::error!("tunnel error: {}", e),
                        Ok(_) => (),
                    },
                    Err(e) => log::error!("upgrade error: {}", e),
                }
            });

            Ok(Response::new(Body::empty()))
        } else {
            log::error!("CONNECT host is not socket addr: {:?}", req.uri());
            let mut resp = Response::new(Body::from("CONNECT must be to a socket address"));
            *resp.status_mut() = http::StatusCode::BAD_REQUEST;

            Ok(resp)
        }
    }

    async fn direct_http(&mut self, req: Request<Body>) -> Result<Response<Body>, SessionError> {
        log::debug!("request {:?}", req);
        let res = self.direct_client.request(req).await?;
        Ok(res)
    }

    async fn forward_connect(
        &mut self,
        proxy: Uri,
        mut req: Request<Body>,
    ) -> Result<Response<Body>, SessionError> {
        // Make a client CONNECT request to the parent proxy to upgrade the connection
        //let uri: http::Uri = proxy.as_str().parse().unwrap(); //.map_err(|_| hyper::Error::new(hyper::error::Kind::User(hyper::error::User::Service))?;
        let host = if let Some(host) = req.headers_mut().get(HOST) {
            host.clone()
        } else {
            HeaderValue::from_str(req.uri().host().unwrap()).unwrap()
        };

        let (req_parts, req_body) = req.into_parts();
        assert_eq!(req_parts.method, http::Method::CONNECT);

        let mut req = Request::connect(req_parts.uri)
            .version(http::version::Version::HTTP_11 /*req_parts.version*/)
            .body(Body::empty())
            .unwrap();
        req.headers_mut().insert(HOST, host);

        log::debug!("forward_connect req: {:?}", req);
        let parent_res = self.proxy_client(proxy).await.request(req).await?;

        if parent_res.status() == StatusCode::OK {
            // Upgrade connection to parent proxy
            match parent_res.into_body().on_upgrade().await {
                Ok(parent_upgraded) => {
                    log::debug!("parent_upgraded: {:?}", parent_upgraded);
                    // On a successful upgrade to the parent proxy, upgrade the
                    // request of the client (the original request maker)
                    tokio::task::spawn(async move {
                        match req_body.on_upgrade().await {
                            Ok(client_upgraded) => {
                                log::debug!("client_upgraded: {:?}", client_upgraded);
                                match tunnel(parent_upgraded, client_upgraded).await {
                                    Err(e) => log::error!("tunnel error: {}", e),
                                    Ok(_) => (),
                                }
                            }
                            Err(e) => log::error!("upgrade error: {}", e),
                        }
                    });
                    // Response with a OK to the client
                    Ok(Response::new(Body::empty()))
                }
                Err(e) => bad_request(&format!("upgrade failed: {}", e)),
            }
        } else {
            bad_request("CONNECT failed")
        }
    }

    async fn forward_http(
        &mut self,
        proxy: http::Uri,
        req: Request<Body>,
    ) -> Result<Response<Body>, SessionError> {
        let res = self.proxy_client(proxy).await.request(req).await?;
        Ok(res)
    }

    pub async fn process(
        &mut self,
        mut req: hyper::Request<Body>,
    ) -> Result<Response<Body>, SessionError> {
        let proxy = self.find_proxy(&req.uri()).await;
        let is_connect = req.method() == hyper::Method::CONNECT;

        log::info!("{} {} via {}", req.method(), req.uri(), proxy);

        let proxy_connection = http::header::HeaderName::from_static("proxy-connection");

        let _proxy_auth = req.headers_mut().remove(http::header::PROXY_AUTHORIZATION);
        let _proxy_conn = req.headers_mut().remove(proxy_connection);

        match (proxy, is_connect) {
            (ProxyDesc::Direct, true) => self.direct_connect(req).await,
            (ProxyDesc::Direct, false) => self.direct_http(req).await,
            (ProxyDesc::Proxy(proxy), true) => self.forward_connect(proxy, req).await,
            (ProxyDesc::Proxy(proxy), false) => self.forward_http(proxy, req).await,
        }
    }
}

async fn dial(uri: &http::Uri) -> std::io::Result<TcpStream> {
    log::debug!("uri {:?}", uri);
    match (uri.host(), uri.port_u16()) {
        (Some(host), Some(port)) => TcpStream::connect((host, port)).await,
        (_, _) => Err(Error::new(ErrorKind::AddrNotAvailable, "invalid URI")),
    }
}

// Bidirectionl copy two async streams
async fn tunnel<T1, T2>(server: T1, client: T2) -> std::io::Result<()>
where
    T1: AsyncRead + AsyncWrite,
    T2: AsyncRead + AsyncWrite,
{
    // Proxying data
    let amounts = {
        let (mut server_rd, mut server_wr) = tokio::io::split(server);
        let (mut client_rd, mut client_wr) = tokio::io::split(client);

        let client_to_server = tokio::io::copy(&mut client_rd, &mut server_wr);
        let server_to_client = tokio::io::copy(&mut server_rd, &mut client_wr);

        try_join(client_to_server, server_to_client).await
    };

    // Print message when done
    match amounts {
        Ok((from_client, from_server)) => {
            log::trace!(
                "client wrote {} bytes and received {} bytes",
                from_client,
                from_server
            );
        }
        Err(e) => {
            log::error!("tunnel error: {:?}", e);
        }
    };
    Ok(())
}

fn bad_request(slice: &str) -> Result<Response<Body>, SessionError> {
    let mut resp = Response::new(Body::from(String::from(slice)));
    *resp.status_mut() = http::StatusCode::BAD_REQUEST;

    Ok(resp)
}

fn make_error_response<E>(error: &E) -> Response<Body>
where
    E: StdError + Send + Sync,
{
    let body = format!(
        include_str!("../500.html"),
        error,
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    let mut resp = Response::new(Body::from(body));
    resp.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::header::HeaderValue::from_static("text/html"),
    );
    *resp.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;

    resp
}

impl Service<Request<Body>> for DetoxSession {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut detox = self.clone();

        let resp = async move {
            let resp = detox.process(req).await;
            log::trace!("response {:?}", resp);
            let out = match resp {
                Err(ref error) => make_error_response(error),
                Ok(resp) => resp,
            };
            Ok(out)
        };
        Box::pin(resp)
    }
}
