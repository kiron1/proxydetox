pub mod http_proxy_connector;
pub mod http_proxy_stream;

use http::{header::HOST, HeaderValue, Request, Response, StatusCode};
pub use http_proxy_connector::HttpProxyConnector;
use hyper::Body;
use std::{future::Future, pin::Pin};

#[derive(Clone, Debug)]
pub struct Client {
    inner: hyper::Client<HttpProxyConnector, Body>,
    extra: hyper::HeaderMap,
}

impl Client {
    pub fn new(client: hyper::Client<HttpProxyConnector, Body>, extra: hyper::HeaderMap) -> Self {
        Self {
            inner: client,
            extra,
        }
    }

    pub fn send(&self, mut req: hyper::Request<Body>) -> hyper::client::ResponseFuture {
        req.headers_mut().extend(self.extra.clone());
        self.inner.request(req)
    }
}

type ResponseFuture = Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send>>;

pub trait ForwardClient {
    fn connect(&self, req: hyper::Request<Body>) -> ResponseFuture;
    fn http(&self, req: hyper::Request<Body>) -> ResponseFuture;
}

impl ForwardClient for hyper::Client<hyper::client::HttpConnector, Body> {
    fn connect(&self, req: http::Request<Body>) -> ResponseFuture {
        let resp = async move {
            if let Ok(stream) = crate::net::dial(req.uri()).await {
                tokio::task::spawn(async move {
                    match req.into_body().on_upgrade().await {
                        Ok(upgraded) => {
                            if let Err(e) = crate::io::tunnel(upgraded, stream).await {
                                log::error!("tunnel error: {}", e)
                            }
                        }
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
        };
        Box::pin(resp)
    }

    fn http(&self, req: http::Request<Body>) -> ResponseFuture {
        let this = self.clone();
        let resp = async move { this.request(req).await };
        Box::pin(resp)
    }
}

impl ForwardClient for Client {
    fn connect(&self, mut req: http::Request<Body>) -> ResponseFuture {
        let this = self.clone();

        let resp = async move {
            // Make a client CONNECT request to the parent proxy to upgrade the connection
            let host = if let Some(host) = req.headers_mut().get(HOST) {
                host.clone()
            } else {
                let host = req.uri().host().expect("uri with host");
                HeaderValue::from_str(host).unwrap()
            };

            let (req_parts, req_body) = req.into_parts();
            assert_eq!(req_parts.method, http::Method::CONNECT);

            let mut req = Request::connect(req_parts.uri.clone())
                .version(http::version::Version::HTTP_11)
                .body(Body::empty())
                .unwrap();
            req.headers_mut().insert(HOST, host);
            req.headers_mut().extend(this.extra);

            log::debug!("forward_connect req: {:?}", req);
            let parent_res = this.inner.request(req).await?;

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
                                    if let Err(e) =
                                        crate::io::tunnel(parent_upgraded, client_upgraded).await
                                    {
                                        log::error!("tunnel error: {}", e)
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
        };
        Box::pin(resp)
    }

    fn http(&self, mut req: hyper::Request<Body>) -> ResponseFuture {
        let this = self.clone();
        let resp = async move {
            req.headers_mut().extend(this.extra);
            log::debug!("forward_http req: {:?}", req);
            let res = this.inner.request(req).await?;
            Ok(res)
        };
        Box::pin(resp)
    }
}

fn bad_request(slice: &str) -> Result<Response<Body>, hyper::Error> {
    let mut resp = Response::new(Body::from(String::from(slice)));
    *resp.status_mut() = http::StatusCode::BAD_REQUEST;
    Ok(resp)
}
