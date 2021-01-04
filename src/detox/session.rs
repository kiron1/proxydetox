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

use http::header::{PROXY_AUTHORIZATION, VIA};
use http::HeaderValue;
use http::Uri;
use http::{Request, Response};
use hyper::service::Service;
use hyper::Body;
use tokio::sync::Mutex;
use tracing_attributes::instrument;

use crate::auth::AuthStore;
use crate::client::HttpProxyConnector;
use paclib::proxy::ProxyDesc;
use paclib::Evaluator;

#[derive(Debug)]
pub enum SessionError {
    Io(std::io::Error),
    Hyper(hyper::Error),
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

impl From<hyper::Error> for SessionError {
    fn from(cause: hyper::Error) -> SessionError {
        SessionError::Hyper(cause)
    }
}

impl From<std::io::Error> for SessionError {
    fn from(cause: std::io::Error) -> SessionError {
        SessionError::Io(cause)
    }
}

type ProxyClient = crate::client::Client;

#[derive(Clone)]
pub struct DetoxSession {
    eval: Arc<Mutex<paclib::Evaluator>>,
    direct_client: hyper::Client<hyper::client::HttpConnector>,
    proxy_clients: Arc<Mutex<HashMap<Uri, ProxyClient>>>,
    auth: Arc<Mutex<AuthStore>>,
}

impl std::fmt::Debug for DetoxSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DetoxSession").finish()
    }
}

impl DetoxSession {
    pub fn new(pac_script: &str) -> Self {
        let eval = Arc::new(Mutex::new(Evaluator::new(pac_script).unwrap()));
        let direct_client = Default::default();
        let proxy_clients = Default::default();
        let auth_store = AuthStore::new().unwrap();
        let auth = Arc::new(Mutex::new(auth_store));
        Self {
            eval,
            direct_client,
            proxy_clients,
            auth,
        }
    }

    // For now just use the first reportet proxy
    async fn find_proxy(&mut self, uri: &http::Uri) -> paclib::proxy::ProxyDesc {
        self.eval.lock().await.find_proxy(&uri).unwrap().first()
    }

    async fn proxy_client(&mut self, uri: http::Uri) -> ProxyClient {
        let mut proxies = self.proxy_clients.lock().await;
        match proxies.get(&uri) {
            Some(proxy) => proxy.clone(),
            None => {
                let mut headers = hyper::HeaderMap::new();
                if let Some(auth) = self.auth.lock().await.find(&uri.host().unwrap()) {
                    tracing::debug!("auth for {:?}", uri.host());
                    let auth = HeaderValue::from_str(&auth.as_basic()).unwrap();
                    headers.insert(PROXY_AUTHORIZATION, auth);
                } else {
                    tracing::debug!("no auth for {:?}", uri.host());
                }

                let client = hyper::Client::builder().build(HttpProxyConnector::new(uri.clone()));
                let client = ProxyClient::new(client, headers);
                proxies.insert(uri, client.clone());
                client
            }
        }
    }

    #[instrument(skip(req), fields(method=?req.method(), uri=?req.uri()))]
    pub async fn process(
        &mut self,
        req: hyper::Request<Body>,
    ) -> Result<Response<Body>, SessionError> {
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

    pub async fn dispatch(
        &mut self,
        mut req: hyper::Request<Body>,
    ) -> Result<Response<Body>, SessionError> {
        let proxy = self.find_proxy(&req.uri()).await;
        let is_connect = req.method() == hyper::Method::CONNECT;

        tracing::info!(%proxy);

        let _proxy_auth = req.headers_mut().remove(http::header::PROXY_AUTHORIZATION);

        let proxy_client;
        let client: &(dyn crate::client::ForwardClient + Send + Sync) = match proxy {
            ProxyDesc::Direct => &self.direct_client,
            ProxyDesc::Proxy(proxy) => {
                proxy_client = self.proxy_client(proxy).await;
                &proxy_client
            }
        };

        let mut res = if is_connect {
            client.connect(req).await
        } else {
            client.http(req).await
        };

        if let Ok(ref mut res) = res {
            let via = HeaderValue::from_str(&format!(
                "{}/{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .unwrap();
            res.headers_mut().insert(VIA, via);
        }
        res.map_err(SessionError::Hyper)
    }

    pub async fn management(
        &mut self,
        _req: hyper::Request<Body>,
    ) -> Result<Response<Body>, SessionError> {
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
