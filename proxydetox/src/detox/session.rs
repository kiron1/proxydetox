use std::convert::Infallible;
use std::error::Error as StdError;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;
use std::task::{self, Poll};

use futures_util::future::try_join;
use http::header::HOST;
use http::HeaderValue;
use http::{Request, Response, StatusCode};
use hyper::service::Service;
use hyper::upgrade::Upgraded;
use hyper::Body;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::net::ToSocketAddrs;

use crate::client::HttpProxyConnector;
use paclib::proxy::ProxyDesc;
use paclib::Evaluator;

#[derive(Clone)]
pub struct DetoxSession {
    inner: Arc<Mutex<DetoxSessionInner>>,
}

impl DetoxSession {
    pub fn new(
        eval: Arc<Mutex<Evaluator>>,
        client: hyper::Client<hyper::client::HttpConnector>,
    ) -> Result<Self, CreateDetoxError> {
        let inner = Arc::new(Mutex::new(
            DetoxSessionInner::new(eval.clone(), client).unwrap(),
        ));
        Ok(Self { inner })
    }
}

struct DetoxSessionInner {
    client: hyper::Client<hyper::client::HttpConnector>,
    eval: Arc<Mutex<paclib::Evaluator>>,
}

impl DetoxSessionInner {
    pub fn new(
        eval: Arc<Mutex<Evaluator>>,
        client: hyper::Client<hyper::client::HttpConnector>,
    ) -> Result<Self, CreateDetoxError> {
        Ok(Self { client, eval })
    }

    // For now just support one single proxy
    async fn find_proxy(&mut self, uri: &http::Uri) -> paclib::proxy::ProxyDesc {
        log::debug!("find proxy for {:?}", &uri);
        let url = format!(
            "{}://{}{}{}",
            uri.scheme().unwrap_or(&http::uri::Scheme::HTTP),
            uri.host().expect("uri with host"),
            uri.port().map(|_| ":").unwrap_or(""),
            uri.port().map(|p| p.to_string()).unwrap_or_default()
        )
        .parse()
        .expect("should be valid Url");
        self.eval.lock().await.find_proxy(&url).unwrap().first()
    }

    async fn direct_connect(&mut self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
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
        if let Some(addr) = host_addr(req.uri()).await {
            tokio::task::spawn(async move {
                match req.into_body().on_upgrade().await {
                    Ok(upgraded) => {
                        log::debug!("connect {:?}", addr);
                        if let Err(e) = tunnel(upgraded, addr).await {
                            log::debug!("server io error: {}", e);
                        };
                    }
                    Err(e) => log::debug!("upgrade error: {}", e),
                }
            });

            Ok(Response::new(Body::empty()))
        } else {
            log::debug!("CONNECT host is not socket addr: {:?}", req.uri());
            let mut resp = Response::new(Body::from("CONNECT must be to a socket address"));
            *resp.status_mut() = http::StatusCode::BAD_REQUEST;

            Ok(resp)
        }
    }

    async fn direct_http(&mut self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        log::debug!("requst {:?}", req);
        self.client.request(req).await
    }

    async fn forward_connect(
        &mut self,
        proxy: url::Url,
        mut req: Request<Body>,
    ) -> Result<Response<Body>, hyper::Error> {
        // Make a client CONNECT request to the parent proxy to upgrade the connection
        //let uri: http::Uri = proxy.as_str().parse().unwrap(); //.map_err(|_| hyper::Error::new(hyper::error::Kind::User(hyper::error::User::Service))?;
        let host = if let Some(host) = req.headers_mut().get(HOST) {
            host.clone()
        } else {
            HeaderValue::from_str(req.uri().host().unwrap()).unwrap()
        };

        let (req_parts, req_body) = req.into_parts();
        assert_eq!(req_parts.method, http::Method::CONNECT);

        let mut req = Request::connect(req_parts.uri).version(http::version::Version::HTTP_11/*req_parts.version*/).body(Body::empty()).unwrap();
        //req.headers_mut().extend(req_parts.headers.iter());
        req.headers_mut().insert(HOST, host);
        // let mut parent_req = hyper::Request::connect(uri).body(Body::empty()).unwrap();
        // parent_req.headers_mut().insert(HOST, host);

        log::debug!("forward_connect req: {:?}", req);
        let client = hyper::Client::builder().build(HttpProxyConnector::new(proxy));
        let parent_res = client.request(req).await?;

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

                                tunnel2(parent_upgraded, client_upgraded).await;
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

        // hyper::Request::connect() will not work, see
        // https://github.com/tafia/hyper-proxy/blob/master/src/tunnel.rs#L62
        // need to create a raw socket and send the CONNECT request.
        // -- OR --
        // Maybe this will work:
        // https://github.com/hyperium/hyper/blob/master/examples/upgrades.rs
        // https://docs.rs/hyper/0.13.7/hyper/upgrade/struct.Upgraded.html
        //
        //let parent_connect = hyper::Request::connect(uri).body(Body::empty()).unwrap();
        //let parent_response = self.client.request(parent_connect).await.unwrap();
        //log::debug!("CONNECT parent status {:?}", parent_response);
    }

    async fn forward_http(
        &mut self,
        proxy: url::Url,
        req: Request<Body>,
    ) -> Result<Response<Body>, hyper::Error> {
        let client = hyper::Client::builder().build(HttpProxyConnector::new(proxy));
        client.request(req).await
    }

    pub async fn process(
        &mut self,
        mut req: hyper::Request<Body>,
    ) -> Result<Response<Body>, hyper::Error> {
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

async fn host_addr(uri: &http::Uri) -> Option<SocketAddr> {
    log::debug!("uri {:?}", uri);
    if uri.authority().is_none() {
        return None;
    }
    let authority = uri.authority().unwrap();
    if authority.port().is_none() {
        return None;
    }

    let host = authority.host();
    let port = authority.port().unwrap().as_u16();

    let host = net::lookup_host(format!("{}:{}", host, port))
        .await
        .ok()
        .and_then(|mut x| x.next());
    host
}

// Create a TCP connection to host:port, build a tunnel between the connection and
// the upgraded connection
async fn tunnel2<T1, T2>(server: T1, client: T2) -> std::io::Result<()>
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

// Create a TCP connection to host:port, build a tunnel between the connection and
// the upgraded connection
async fn tunnel(upgraded: Upgraded, addr: SocketAddr) -> std::io::Result<()> {
    // Connect to remote server
    let mut server = TcpStream::connect(addr).await?;

    // Proxying data
    let amounts = {
        let (mut server_rd, mut server_wr) = server.split();
        let (mut client_rd, mut client_wr) = tokio::io::split(upgraded);

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
            log::info!("tunnel error: {}", e);
        }
    };
    Ok(())
}

fn bad_request(slice: &str) -> Result<Response<Body>, hyper::Error> {
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

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let detox = self.inner.clone();

        let resp = async move {
            let resp = detox.lock().await.process(req).await;
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CreateDetoxError;

impl std::error::Error for CreateDetoxError {}

impl std::fmt::Display for CreateDetoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "create detox failed")
    }
}
