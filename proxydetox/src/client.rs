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

use crate::auth::Authenticator;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP connection upgrade with {1} failed: {0}")]
    UpgradeFailed(#[source] hyper::Error, http::Uri),
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("authentication mechanism error: {0}")]
    Auth(#[from] crate::auth::Error),
    #[error("invalid URI")]
    InvalidUri,
    #[error("response already taken")]
    ResponseAlreadyTaken,
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("invalid URI: {0}")]
    InvalidUri(http::Uri),
    #[error("request to {0} failed: {1}")]
    RequestFailed(http::Uri, #[source] Error),
    #[error("HTTP CONNECT to {1} failed, status {0}")]
    ConnectFailed(http::StatusCode, http::Uri),
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
                .map(|i| i.remote_addr);
            tracing::error!(?remote_addr, "proxy requires authentication");
            self.0.requires_auth.store(true, Ordering::Relaxed);
        }
        Ok(res)
    }

    pub async fn connect(
        &self,
        uri: http::Uri,
    ) -> std::result::Result<ConnectHandle, ConnectError> {
        // Make a client CONNECT request to the parent proxy to upgrade the connection
        let parent_authority = authority_of(&uri).map_err({
            let uri = uri.clone();
            move |_| ConnectError::InvalidUri(uri)
        })?;
        let parent_req = Request::connect(parent_authority.clone())
            .version(http::version::Version::HTTP_11)
            .body(Body::empty())
            .unwrap();

        let parent_res = self.request(parent_req).await.map_err({
            let uri = uri.clone();
            move |e| ConnectError::RequestFailed(uri, e)
        })?;

        if parent_res.status() == StatusCode::OK {
            Ok(ConnectHandle::new(parent_authority, parent_res))
        } else {
            Err(ConnectError::ConnectFailed(parent_res.status(), uri))
        }
    }
}

impl tower::Service<Request<Body>> for ProxyClient {
    type Response = Response<Body>;
    type Error = Error;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let this = self.clone();
        let res = async move { this.request(req).await };
        Box::pin(res)
    }
}

#[derive(Debug, Clone)]
pub struct ConnectHandle {
    uri: http::Uri,
    response: Arc<Mutex<Option<Response<Body>>>>,
}

impl ConnectHandle {
    pub fn new(uri: http::Uri, res: http::Response<Body>) -> Self {
        Self {
            uri,
            response: Arc::new(Mutex::new(Some(res))),
        }
    }
}

impl tower::Service<Request<Body>> for ConnectHandle {
    type Response = Response<Body>;
    type Error = Error;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let uri = self.uri.clone();
        let response = {
            let mut gurad = self.response.lock();
            gurad.take()
        };
        let res = async move {
            if let Some(response) = response {
                // Upgrade connection to parent proxy
                match hyper::upgrade::on(response).await {
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
                                        tracing::error!(%cause, "tunnel error")
                                    }
                                }
                                Err(cause) => tracing::error!(%cause, "upgrade error"),
                            }
                        });
                        // Response with a OK to the client
                        Ok(Response::new(Body::empty()))
                    }
                    Err(cause) => Err(Error::UpgradeFailed(cause, uri)),
                }
            } else {
                tracing::error!("response already taken");
                Err(Error::ResponseAlreadyTaken)
            }
        };
        Box::pin(res)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("invalid URI")]
struct InvalidUri;

fn authority_of(uri: &http::Uri) -> std::result::Result<http::Uri, InvalidUri> {
    match (uri.scheme(), uri.host(), uri.port()) {
        (Some(scheme), Some(host), None) => {
            let port = if *scheme == http::uri::Scheme::HTTP {
                "80"
            } else if *scheme == http::uri::Scheme::HTTPS {
                "443"
            } else {
                return Err(InvalidUri);
            };
            let uri = format!("{}:{}", host, port);
            uri.parse().map_err(|_| InvalidUri)
        }
        (_, Some(host), Some(port)) => {
            let uri = format!("{}:{}", host, port);
            uri.parse().map_err(|_| InvalidUri)
        }
        (_, _, _) => Err(InvalidUri),
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
