pub mod builder;

use crate::accesslog;
use detox_auth::AuthenticatorFactory;
use detox_futures::FutureExt;
use detox_hyper::conn::Connection;
use detox_net::{HostAndPort, PathOrUri, TcpKeepAlive};
use http::Uri;
use paclib::ProxyOrDirect;
use std::fs::read_to_string;
use std::future::IntoFuture;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::broadcast::Sender;
use tracing::field::debug;
// use tracing::Instrument;
use tracing_attributes::instrument;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid URI: {0}")]
    InvalidUri(Uri),
    #[error("invalid host: {0}")]
    InvalidHost(
        #[source]
        #[from]
        detox_net::host_and_port::Error,
    ),
    #[error("timeout when connecting to {1} via proxy {0}")]
    ConnectTimeout(ProxyOrDirect, Uri),
    #[error("client error: {0}")]
    Client(
        #[from]
        #[source]
        hyper::Error,
    ),
    #[error("connect error reaching {2} via {1}: {0}")]
    Connect(#[source] tokio::io::Error, ProxyOrDirect, Uri),
    #[error("upstream proxy ({0}) requires authentication")]
    ProxyAuthenticationRequired(ProxyOrDirect),
    #[error("http error: {0}")]
    Http(
        #[source]
        #[from]
        http::Error,
    ),
    #[error("unable to establish connection: {0}")]
    UnableToEstablishConnection(Uri),
    #[error("handshake error")]
    Handshake,
}

/// Shared context between the servcie (one server instance) and all sessions (for each peer).
pub struct Context {
    pub(super) eval: paclib::Evaluator,
    pub(super) auth: AuthenticatorFactory,
    pub(super) proxytunnel: bool,
    pub(super) race_connect: bool,
    pub(super) parallel_connect: usize,
    pub(super) direct_fallback: bool,
    pub(super) tls_config: Arc<rustls::ClientConfig>,
    pub(super) connect_timeout: Duration,
    pub(super) client_tcp_keepalive: TcpKeepAlive,
    pub(super) accesslog_tx: Sender<accesslog::Entry>,
}

impl Context {
    pub fn builder() -> builder::Builder {
        Default::default()
    }

    pub(super) async fn find_proxy(&self, uri: Uri) -> paclib::Proxies {
        let mut proxies = self
            .eval
            .find_proxy(uri.clone())
            .await
            .unwrap_or_else(|cause| {
                tracing::error!(%cause, %uri, "failed to find_proxy");
                paclib::Proxies::direct()
            });
        if self.direct_fallback && !proxies.iter().any(|p| *p == ProxyOrDirect::Direct) {
            proxies.push(ProxyOrDirect::Direct);
        }
        proxies
    }

    #[instrument(skip(self))]
    pub async fn load_pac_file(&self, uri: &Option<PathOrUri>) -> std::io::Result<()> {
        tracing::info!("update PAC script");
        let pac = if let Some(uri) = uri {
            let pac = match uri {
                PathOrUri::Path(p) => read_to_string(p)?,
                PathOrUri::Uri(u) => detox_hyper::http_file(u.clone(), self.tls_config.clone())
                    .timeout(Duration::from_secs(15))
                    .await
                    .map_err(std::io::Error::other)??,
            };
            Some(pac)
        } else {
            None
        };
        self.eval
            .set_pac_script(pac)
            .await
            .map_err(std::io::Error::other)
    }

    /// Establish a connection to parent proxy.
    ///
    /// In case of `CONNECT` the connesction will be established so far that `CONNECT` request is
    /// send, but not the client request.
    /// For upstream servers which can be connected directly a TCP connection will be established.
    #[instrument(level = "debug", skip(self, method, uri), err, fields(proxy = %proxy, duration))]
    pub(super) async fn connect(
        self: Arc<Self>,
        proxy: ProxyOrDirect,
        method: http::Method,
        uri: http::Uri,
    ) -> Result<Connection, Error> {
        let dst = HostAndPort::try_from_uri(&uri)?;
        let tunnel = method == hyper::Method::CONNECT || self.proxytunnel;
        let tls_config = self.tls_config.clone();
        let auth = self.auth.clone();
        let connect_timeout = self.connect_timeout;

        let conn = match proxy {
            ProxyOrDirect::Proxy(ref proxy) => {
                if tunnel {
                    Connection::http_tunnel(proxy.clone(), tls_config.clone(), auth.clone(), dst)
                } else {
                    Connection::http_proxy(proxy.clone(), tls_config.clone(), auth.clone())
                }
            }
            ProxyOrDirect::Direct => Connection::http(dst),
        };
        let conn = conn.with_tcp_keepalive(self.client_tcp_keepalive.clone());

        let start = Instant::now();
        let conn = conn
            .into_future()
            .timeout(connect_timeout * if tunnel { 2 } else { 1 })
            .await
            .map_err({
                let proxy = proxy.clone();
                let uri = uri.clone();
                move |_| Error::ConnectTimeout(proxy, uri)
            })?
            .map_err(move |e| Error::Connect(e, proxy, uri))?;
        tracing::Span::current().record("duration", debug(&start.elapsed()));
        tracing::debug!("connect");

        Ok(conn)
    }
}
