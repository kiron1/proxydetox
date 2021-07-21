#![allow(clippy::type_complexity)]

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::{
    collections::HashMap,
    task::{self, Poll},
};

use http::header::VIA;
use http::HeaderValue;
use http::Uri;
use http::{Request, Response};
use hyper::service::Service;
use hyper::Body;
use tokio::sync::Mutex;
use tracing_attributes::instrument;

use crate::{auth::AuthenticatorFactory, client::HttpProxyConnector};
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
    #[error("upstream proxy ({0}) requires authentication")]
    ProxyAuthenticationRequired(ProxyDesc),
}

impl From<crate::client::Error> for Error {
    fn from(cause: crate::client::Error) -> Error {
        use crate::client;
        match cause {
            client::Error::Auth(cause) => Error::Auth(cause),
            client::Error::Hyper(cause) => Error::Hyper(cause),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

type ProxyClient = crate::client::Client;

#[derive(Clone)]
pub struct DetoxSession {
    eval: Arc<Mutex<paclib::Evaluator>>,
    direct_client: hyper::Client<hyper::client::HttpConnector>,
    proxy_clients: Arc<Mutex<HashMap<Uri, ProxyClient>>>,
    //auth: Arc<Mutex<Authenticator>>,
    auth: AuthenticatorFactory,
    config: super::Config,
}

impl std::fmt::Debug for DetoxSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DetoxSession").finish()
    }
}

impl DetoxSession {
    pub fn new(pac_script: &str, auth: AuthenticatorFactory, config: super::Config) -> Self {
        let eval = Arc::new(Mutex::new(Evaluator::new(pac_script).unwrap()));
        let direct_client = hyper::Client::builder()
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .pool_idle_timeout(config.pool_idle_timeout)
            .build_http();
        let proxy_clients = Default::default();
        Self {
            eval,
            direct_client,
            proxy_clients,
            auth,
            config,
        }
    }

    // For now just use the first reportet proxy
    async fn find_proxy(&mut self, uri: &http::Uri) -> paclib::proxy::ProxyDesc {
        let eval = self.eval.clone();
        let uri = uri.clone();
        let proxy = tokio::task::spawn_blocking(move || {
            futures::executor::block_on(async {
                eval.lock().await.find_proxy(&uri).unwrap_or_else(|cause| {
                    tracing::error!("failed to find_proxy: {:?}", cause);
                    paclib::Proxies::direct()
                })
            })
        })
        .await;
        match proxy {
            Ok(proxy) => proxy,
            Err(cause) => {
                tracing::error!("failed to join: {:?}", cause);
                paclib::Proxies::direct()
            }
        }
        .first()
    }

    async fn proxy_client(&mut self, uri: http::Uri) -> Result<ProxyClient> {
        let mut proxies = self.proxy_clients.lock().await;
        match proxies.get(&uri) {
            Some(proxy) => Ok(proxy.clone()),
            None => {
                tracing::debug!("new proxy client for {:?}", uri.host());
                let auth = self.auth.make(&uri)?;
                let client = hyper::Client::builder()
                    .pool_max_idle_per_host(self.config.pool_max_idle_per_host)
                    .pool_idle_timeout(self.config.pool_idle_timeout)
                    .build(HttpProxyConnector::new(uri.clone()));
                let client = ProxyClient::new(client, auth);
                proxies.insert(uri, client.clone());
                Ok(client)
            }
        }
    }

    #[instrument(skip(req), fields(method=?req.method(), uri=?req.uri()))]
    pub async fn process(&mut self, req: hyper::Request<Body>) -> Result<Response<Body>> {
        let res = if req.uri().authority().is_some() {
            self.dispatch(req).await
        } else if req.method() == hyper::Method::GET {
            self.management(req).await
        } else {
            let mut resp = Response::new(Body::from(String::from("Invalid Requst")));
            *resp.status_mut() = http::StatusCode::BAD_REQUEST;
            Ok(resp)
        };
        res
    }

    pub async fn dispatch(&mut self, mut req: hyper::Request<Body>) -> Result<Response<Body>> {
        let proxy = self.find_proxy(&req.uri()).await;
        let is_connect = req.method() == hyper::Method::CONNECT;

        tracing::info!(%proxy);

        // Remove hop-by-hop headers which are meant for the proxy.
        // "proxy-connection" is not an official header, but used by many clients.
        let _proxy_connection = req
            .headers_mut()
            .remove(http::header::HeaderName::from_static("proxy-connection"));
        let _proxy_auth = req.headers_mut().remove(http::header::PROXY_AUTHORIZATION);

        let proxy_client;
        let client: &(dyn crate::client::ForwardClient + Send + Sync) = match proxy {
            ProxyDesc::Direct => &self.direct_client,
            ProxyDesc::Proxy(ref proxy) => {
                proxy_client = self.proxy_client(proxy.clone()).await?;
                &proxy_client
            }
        };

        let mut res = if is_connect {
            client.connect(req).await
        } else {
            client.http(req).await
        };

        if let Ok(ref mut res) = res {
            if res.status() == http::StatusCode::PROXY_AUTHENTICATION_REQUIRED {
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
        Ok(res?)
    }

    pub async fn management(&mut self, _req: hyper::Request<Body>) -> Result<Response<Body>> {
        let body = format!(
            "<!DOCTYPE html><html><h1>{}/{}</h1><h2>DNS Cache</h2><code>{:?}</code></html>",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            self.eval.lock().await.cache()
        );
        let mut resp = Response::new(Body::from(body));
        resp.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_static("text/html"),
        );
        Ok(resp)
    }
}

fn make_error_response<E>(error: &E) -> Response<Body>
where
    E: std::error::Error + Send + Sync,
{
    let body = format!(
        include_str!("../502.html"),
        error,
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    let mut resp = Response::new(Body::from(body));
    resp.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::header::HeaderValue::from_static("text/html"),
    );
    *resp.status_mut() = http::StatusCode::BAD_GATEWAY;

    resp
}

impl Service<Request<Body>> for DetoxSession {
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
        let mut detox = self.clone();

        let resp = async move {
            let resp = detox.process(req).await;
            tracing::trace!("response {:?}", resp);
            let out = match resp {
                Err(ref error) => make_error_response(error),
                Ok(resp) => resp,
            };
            Ok(out)
        };
        Box::pin(resp)
    }
}
