use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use detox_net::HostAndPort;
use http::Uri;
use http::{Request, Response};
use hyper::Body;
use paclib::ProxyOrDirect;
use proxy_client::HttpProxyConnector;
use tokio::sync::broadcast::Sender;
use tokio::time::timeout;
use tower::{util::BoxService, Service, ServiceExt};
use tracing_attributes::instrument;

use super::Error;
use super::Result;
use crate::accesslog;
use crate::auth::AuthenticatorFactory;
use crate::client::ProxyClient;
use crate::connect::Connect;
use paclib::proxy::Proxy;

pub(crate) struct Shared {
    pub(super) eval: Mutex<paclib::Evaluator>,
    pub(super) direct_client: Mutex<crate::client::Direct>,
    pub(super) proxy_clients: Mutex<HashMap<Proxy, ProxyClient>>,
    pub(super) auth: AuthenticatorFactory,
    pub(super) always_use_connect: bool,
    pub(super) direct_fallback: bool,
    pub(super) connect_timeout: Duration,
    pub(super) accesslog_tx: Sender<accesslog::Entry>,
}

impl Shared {
    pub(super) fn find_proxy(&self, uri: &Uri) -> paclib::Proxies {
        tokio::task::block_in_place(move || {
            self.eval
                .lock()
                .unwrap()
                .find_proxy(uri)
                .unwrap_or_else(|cause| {
                    tracing::error!(%cause, %uri, "failed to find_proxy");
                    paclib::Proxies::direct()
                })
        })
    }

    pub(super) fn proxy_for(&self, proxy: &Proxy) -> Result<ProxyClient> {
        let mut proxies = self.proxy_clients.lock().unwrap();
        match proxies.get(proxy) {
            Some(proxy) => Ok(proxy.clone()),
            None => {
                tracing::debug!(%proxy, "new proxy client");
                let auth = self.auth.make(proxy.host());
                let auth = match auth {
                    Ok(auth) => auth,
                    Err(ref cause) => {
                        tracing::warn!(%cause, "error makeing authenticator");
                        Box::new(crate::auth::NoneAuthenticator)
                    }
                };
                let client = hyper::Client::builder().build(HttpProxyConnector::new(proxy.clone()));
                let client = ProxyClient::new(client, auth);
                proxies.insert(proxy.clone(), client.clone());
                Ok(client)
            }
        }
    }

    #[instrument(level = "trace", skip(self))]
    pub(super) fn proxy_client(
        &self,
        proxy: &Proxy,
    ) -> Result<BoxService<Request<Body>, Response<Body>, Error>> {
        let client = self.proxy_for(proxy);
        client.map(|s| {
            let proxy = proxy.clone();
            s.map_err(move |e| Error::MakeProxyClient(e, proxy)).boxed()
        })
    }

    async fn proxy_connect(
        &self,
        proxy: &Proxy,
        uri: &http::Uri,
    ) -> Result<BoxService<Request<Body>, Response<Body>, Error>> {
        let proxy_client = self.proxy_for(proxy)?;
        let host = HostAndPort::try_from_uri(uri)?;
        let conn = timeout(self.connect_timeout, proxy_client.connect(host)).await;
        let conn = match conn {
            Ok(conn) => conn,
            // Timeout condition
            Err(_) => return Err(Error::ConnectTimeout(proxy.clone())),
        };
        conn.map_err({
            let proxy = proxy.clone();
            let uri = uri.clone();
            move |e| Error::ProxyConnect(e, proxy, uri)
        })
        .map(move |c| {
            let proxy = proxy.clone();
            let uri = uri.clone();
            c.map_err(|e| Error::Upstream(e, proxy, uri)).boxed()
        })
    }

    async fn direct_client(
        &self,
        uri: http::Uri,
    ) -> Result<BoxService<Request<Body>, Response<Body>, Error>> {
        let client = {
            let uri = uri.clone();
            let mut guard = self.direct_client.lock().unwrap();
            guard.call(uri)
        };
        client
            .await
            .map_err(move |e| Error::MakeClient(e, uri))
            .map(|s| s.map_err(Error::Client).boxed())
    }

    /// Establish a connection to parent proxy.
    ///
    /// In case of `CONNECT` the connesction will be established so far that `CONNECT` request is
    /// send, but not the client request.
    /// For upstream servers which can be connected directly a TCP connection will be established.
    /// For a directly reachable server with a regular HTTP request, no action will be perforemd.
    #[instrument(skip(self, method, uri))]
    pub(super) async fn establish_connection(
        &self,
        proxy: ProxyOrDirect,
        method: &http::Method,
        uri: &http::Uri,
    ) -> Result<BoxService<Request<Body>, Response<Body>, Error>> {
        let is_connect = method == hyper::Method::CONNECT;
        let use_connect = self.always_use_connect;

        match (is_connect, use_connect, proxy) {
            (true, _, ProxyOrDirect::Proxy(ref proxy)) => self.proxy_connect(proxy, uri).await,
            (false, true, ProxyOrDirect::Proxy(ref proxy)) => self.proxy_connect(proxy, uri).await,
            (false, false, ProxyOrDirect::Proxy(ref proxy)) => self.proxy_client(proxy),
            (true, _, ProxyOrDirect::Direct) => {
                let mut conn = Connect::new();
                let handshake = conn.call(uri.clone()).await;
                handshake
                    .map_err({
                        let uri = uri.clone();
                        move |e| Error::Connect(e, uri)
                    })
                    .map(|s| s.map_err(|_| Error::Handshake).boxed())
            }
            (false, _, ProxyOrDirect::Direct) => self.direct_client(uri.clone()).await,
        }
    }
}
