pub mod http_proxy_connector;
pub mod http_proxy_stream;

use http::{header::HOST, HeaderValue, Request, Response, StatusCode};
pub use http_proxy_connector::HttpProxyConnector;
use http_proxy_stream::HttpProxyInfo;
use hyper::Body;
use std::{future::Future, pin::Pin};
use tracing_attributes::instrument;

use crate::auth::Authenticator;

#[derive(Clone, Debug)]
pub struct Client {
    inner: hyper::Client<HttpProxyConnector, Body>,
    auth: Authenticator,
}

impl Client {
    pub fn new(client: hyper::Client<HttpProxyConnector, Body>, auth: Authenticator) -> Self {
        Self {
            inner: client,
            auth,
        }
    }

    pub async fn send(&self, mut req: hyper::Request<Body>) -> hyper::Result<Response<Body>> {
        // TODO: fix the unwrap here, need to propagade this any errors here to the user
        let headers = self.auth.step(None).await.unwrap();
        req.headers_mut().extend(headers);
        self.inner.request(req).await
    }
}

type ResponseFuture = Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send>>;

pub trait ForwardClient {
    fn connect(&self, req: hyper::Request<Body>) -> ResponseFuture;
    fn http(&self, req: hyper::Request<Body>) -> ResponseFuture;
}

impl ForwardClient for hyper::Client<hyper::client::HttpConnector, Body> {
    #[instrument]
    fn connect(&self, req: http::Request<Body>) -> ResponseFuture {
        let resp = async move {
            if let Ok(stream) = crate::net::dial(req.uri()).await {
                tokio::task::spawn(async move {
                    match hyper::upgrade::on(req).await {
                        Ok(upgraded) => {
                            if let Err(e) = crate::io::tunnel(upgraded, stream).await {
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
        Box::pin(resp)
    }

    #[instrument]
    fn http(&self, req: http::Request<Body>) -> ResponseFuture {
        let this = self.clone();
        let resp = async move { this.request(req).await };
        Box::pin(resp)
    }
}

impl ForwardClient for Client {
    #[instrument]
    fn connect(&self, mut req: http::Request<Body>) -> ResponseFuture {
        let this = self.clone();
        let uri = req.uri().clone();

        let resp = async move {
            // Make a client CONNECT request to the parent proxy to upgrade the connection
            let host = if let Some(host) = req.headers_mut().get(HOST) {
                host.clone()
            } else {
                let host = req.uri().host().expect("uri with host");
                HeaderValue::from_str(host).unwrap()
            };

            assert_eq!(req.method(), http::Method::CONNECT);

            let mut parent_req = Request::connect(req.uri().clone())
                .version(http::version::Version::HTTP_11)
                .body(Body::empty())
                .unwrap();
            parent_req.headers_mut().insert(HOST, host);

            tracing::debug!("forward_connect req: {:?}", req);
            let parent_res = this.send(parent_req).await?;

            if parent_res.status() == StatusCode::OK {
                let http_proxy_info = parent_res
                    .extensions()
                    .get::<HttpProxyInfo>()
                    .map(|i| i.clone());

                // Upgrade connection to parent proxy
                match hyper::upgrade::on(parent_res).await {
                    Ok(parent_upgraded) => {
                        // On a successful upgrade to the parent proxy, upgrade the
                        // request of the client (the original request maker)
                        tokio::task::spawn(async move {
                            match hyper::upgrade::on(&mut req).await {
                                Ok(client_upgraded) => {
                                    if let Err(cause) =
                                        crate::io::tunnel(parent_upgraded, client_upgraded).await
                                    {
                                        tracing::error!(
                                            ?http_proxy_info,
                                            ?cause,
                                            ?uri,
                                            "tunnel error"
                                        )
                                    }
                                }
                                Err(e) => tracing::error!("upgrade error: {}", e),
                            }
                        });
                        // Response with a OK to the client
                        Ok(Response::new(Body::empty()))
                    }
                    Err(cause) => bad_request(&format!("upgrade failed: {}", cause)),
                }
            } else {
                bad_request("CONNECT failed")
            }
        };
        Box::pin(resp)
    }

    #[instrument]
    fn http(&self, req: hyper::Request<Body>) -> ResponseFuture {
        let this = self.clone();
        let resp = async move {
            tracing::debug!("forward_http req: {:?}", req);
            let res = this.send(req).await?;
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
