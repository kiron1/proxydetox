use detox_net::HostAndPort;
use http::{header::CONNECTION, Request, Response, StatusCode};
use hyper::client::conn;
use hyper::Body;
use proxy_client::HttpProxyInfo;
use std::{
    future::Future,
    pin::Pin,
    sync::Mutex,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tracing::Instrument;

use crate::auth::Authenticator;
use crate::net::copy_bidirectional;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP connection upgrade with {1} failed: {0}")]
    UpgradeFailed(#[source] hyper::Error, HostAndPort),
    #[error("HTTP error: {0}")]
    Hyper(
        #[from]
        #[source]
        hyper::Error,
    ),
    #[error("authentication mechanism error: {0}")]
    Auth(
        #[from]
        #[source]
        crate::auth::Error,
    ),
    #[error("invalid URI")]
    InvalidUri,
    #[error("response already taken")]
    ResponseAlreadyTaken,
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("invalid URI `{0}`: {1}")]
    InvalidUri(#[source] detox_net::host_and_port::Error, http::Uri),
    #[error("request to {0} failed: {1}")]
    RequestFailed(HostAndPort, #[source] Error),
    #[error("HTTP CONNECT to {1} failed, status {0}")]
    ConnectFailed(http::StatusCode, HostAndPort),
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
        let auth = self.auth.lock().unwrap();
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
        let auth_error = if self.0.requires_auth.load(Ordering::Relaxed) {
            self.0
                .auth_step(None)
                .await
                .map(|h| req.headers_mut().extend(h))
                .err()
        } else {
            None
        };
        let res = self.0.client.request(req).await?;
        if res.status() == http::StatusCode::PROXY_AUTHENTICATION_REQUIRED {
            let remote_addr = res
                .extensions()
                .get::<HttpProxyInfo>()
                .map(|i| i.remote_addr);
            tracing::error!(?remote_addr, ?auth_error, "proxy requires authentication");
            self.0.requires_auth.store(true, Ordering::Relaxed);
        }
        Ok(res)
    }

    pub async fn connect(
        &self,
        host: HostAndPort,
    ) -> std::result::Result<ConnectHandle, ConnectError> {
        // Make a client CONNECT request to the parent proxy to upgrade the connection
        let parent_req = Request::connect(host.clone())
            .version(http::version::Version::HTTP_11)
            .body(Body::empty())
            .unwrap();

        let parent_res = self.request(parent_req).await.map_err({
            let host = host.clone();
            move |e| ConnectError::RequestFailed(host, e)
        })?;

        if parent_res.status() == StatusCode::OK {
            Ok(ConnectHandle::new(host, parent_res))
        } else {
            Err(ConnectError::ConnectFailed(parent_res.status(), host))
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

#[derive(Debug)]
pub struct ConnectHandle {
    host: HostAndPort,
    response: Option<Response<Body>>,
}

impl ConnectHandle {
    pub fn new(host: HostAndPort, res: http::Response<Body>) -> Self {
        Self {
            host,
            response: Some(res),
        }
    }
}

/// Maps a CONNECT request to an upstream CONNECT reuqest.
async fn upgrade_downstream_upstream(
    host: HostAndPort,
    upstream_response: Response<Body>,
    mut req: Request<Body>,
) -> Result<Response<Body>> {
    // Upgrade connection to parent proxy
    match hyper::upgrade::on(upstream_response).await {
        Ok(mut parent_upgraded) => {
            // On a successful upgrade to the parent proxy, upgrade the
            // request of the client (the original request maker)
            let upgrade_task = async move {
                match hyper::upgrade::on(&mut req).await {
                    Ok(mut client_upgraded) => {
                        copy_bidirectional(&mut parent_upgraded, &mut client_upgraded)
                            .await
                            .ok();
                    }
                    Err(cause) => tracing::error!(%cause, "upgrade error"),
                }
            };
            tokio::task::spawn(upgrade_task.instrument(tracing::info_span!("proxy connect")));
            // Response with a OK to the client
            Ok(Response::new(Body::empty()))
        }
        Err(cause) => Err(Error::UpgradeFailed(cause, host.clone())),
    }
}

/// Maps a HTTP proxy request to an upstream CONNECT request.
async fn upgrade_upstream(
    host: HostAndPort,
    upstream_response: Response<Body>,
    mut req: Request<Body>,
) -> Result<Response<Body>> {
    // Upgrade connection to parent proxy
    match hyper::upgrade::on(upstream_response).await {
        Ok(parent_upgraded) => {
            let (mut request_sender, connection) = conn::handshake(parent_upgraded).await?;
            // spawn a client which uses the established CONNECT stream
            tokio::spawn(async move {
                if let Err(cause) = connection.await {
                    tracing::error!(%cause, %host, "error in upgraded upstream connection");
                }
            });

            *req.uri_mut() = http::Uri::builder()
                .path_and_query(
                    req.uri()
                        .path_and_query()
                        .cloned()
                        .unwrap_or_else(|| http::uri::PathAndQuery::from_static("/")),
                )
                .build()
                .unwrap();
            let mut res = request_sender.send_request(req).await?;
            res.headers_mut()
                .insert(CONNECTION, "close".parse().unwrap());

            Ok(res)
        }
        Err(cause) => Err(Error::UpgradeFailed(cause, host.clone())),
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

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let host = self.host.clone();
        let response = self.response.take();
        let res = async move {
            if let Some(response) = response {
                let proxy_addr = response
                    .extensions()
                    .get::<HttpProxyInfo>()
                    .map(|i| i.remote_addr);
                if req.method() == http::method::Method::CONNECT {
                    upgrade_downstream_upstream(host, response, req)
                        .instrument(tracing::info_span!(
                            "upgrade_downstream_upstream",
                            ?proxy_addr
                        ))
                        .await
                } else {
                    upgrade_upstream(host, response, req).await
                }
            } else {
                tracing::error!("response already taken");
                Err(Error::ResponseAlreadyTaken)
            }
        };
        Box::pin(res)
    }
}
