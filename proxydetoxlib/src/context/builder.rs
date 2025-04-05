use super::Context;
use detox_auth::AuthenticatorFactory;
use detox_net::{PathOrUri, TcpKeepAlive};
use paclib::Evaluator;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

#[derive(Debug, Default)]
pub struct Builder {
    pac_file: Option<PathOrUri>,
    pac_script: Option<String>,
    auth: Option<AuthenticatorFactory>,
    proxytunnel: bool,
    direct_fallback: bool,
    tls_config: Option<Arc<rustls::ClientConfig>>,
    connect_timeout: Option<Duration>,
    race_connect: bool,
    parallel_connect: usize,
    client_tcp_keepalive: TcpKeepAlive,
}

impl Builder {
    /// PAC script URI to be loaded and used for evaluation
    /// If `None`, FindProxy will evaluate to DIRECT
    pub fn pac_file(mut self, uri: Option<PathOrUri>) -> Self {
        self.pac_file = uri;
        self
    }

    /// PAC script to use
    /// If `None`, FindProxy will evaluate to DIRECT
    pub fn pac_script(mut self, script: String) -> Self {
        self.pac_script = Some(script);
        self
    }
    /// Authenticator factory (Basic or Negotiate)
    /// If `None`, use no authentication toward the proxy.
    pub fn authenticator_factory(mut self, factory: Option<AuthenticatorFactory>) -> Self {
        self.auth = factory;
        self
    }

    /// use the CONNECT method even for HTTP requests.
    pub fn proxytunnel(mut self, yesno: bool) -> Self {
        self.proxytunnel = yesno;
        self
    }

    /// use DIRECT when connecting the proxies failed
    pub fn direct_fallback(mut self, yesno: bool) -> Self {
        self.direct_fallback = yesno;
        self
    }

    /// TLS configuration to use when connecting to HTTPS servers or proxies.
    pub fn tls_config(mut self, tls_config: Arc<rustls::ClientConfig>) -> Self {
        self.tls_config = Some(tls_config);
        self
    }

    /// Timeout to use when trying to estabish a new connection.
    pub fn connect_timeout(mut self, duration: Duration) -> Self {
        self.connect_timeout = Some(duration);
        self
    }

    /// Race connects in parallel.
    pub fn race_connect(mut self, race: bool) -> Self {
        self.race_connect = race;
        self
    }

    /// Number of connects to run in parallel.
    pub fn parallel_connect(mut self, num: usize) -> Self {
        self.parallel_connect = num;
        self
    }
    /// TCP keep alive settings for client sockets.
    pub fn client_tcp_keepalive(mut self, keepalive: TcpKeepAlive) -> Self {
        self.client_tcp_keepalive = keepalive;
        self
    }

    pub fn build(self) -> Arc<Context> {
        let auth = self.auth.unwrap_or(AuthenticatorFactory::None);
        let eval = if let Some(pac) = self.pac_script {
            Evaluator::with_pac_script(&pac).unwrap_or_default()
        } else {
            Evaluator::new()
        };
        let tls_config = self.tls_config.unwrap_or_else(default_tls_config);
        let (accesslog_tx, mut accesslog_rx) = broadcast::channel(16);
        tokio::spawn(async move {
            loop {
                let entry = accesslog_rx.recv().await;
                if let Err(cause) = entry {
                    if cause == broadcast::error::RecvError::Closed {
                        break;
                    }
                }
            }
        });
        let context = Context {
            eval,
            auth,
            proxytunnel: self.proxytunnel,
            race_connect: self.race_connect,
            parallel_connect: self.parallel_connect.max(1),
            direct_fallback: self.direct_fallback,
            tls_config,
            connect_timeout: self.connect_timeout.unwrap_or(Duration::new(30, 0)),
            client_tcp_keepalive: self.client_tcp_keepalive,
            accesslog_tx,
        };
        let context = Arc::new(context);

        if self.pac_file.is_some() {
            tokio::spawn({
                let context = context.clone();
                async move {
                    if let Err(cause) = context.load_pac_file(&self.pac_file).await {
                        tracing::error!(%cause, pac_file = ?&self.pac_file, "failed to load PAC from URI");
                    }
                }
            });
        }

        context
    }
}

fn default_tls_config() -> Arc<rustls::ClientConfig> {
    let mut roots = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().expect("load platform certs") {
        roots.add(cert).unwrap();
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    Arc::new(config)
}
