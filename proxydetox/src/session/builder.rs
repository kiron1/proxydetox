use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::broadcast;

use super::Session;
use super::Shared;
use crate::auth::AuthenticatorFactory;
use paclib::Evaluator;

#[derive(Debug, Default)]
pub struct Builder {
    pac_script: Option<String>,
    auth: Option<AuthenticatorFactory>,
    always_use_connect: bool,
    direct_fallback: bool,
    connect_timeout: Option<Duration>,
}

impl Builder {
    /// PAC script used for evaluation
    /// If `None`, FindProxy will evaluate to DIRECT
    pub fn pac_script(mut self, pac_script: Option<String>) -> Self {
        self.pac_script = pac_script;
        self
    }
    /// Authenticator factory (Basic or Negotiate)
    /// If `None`, use no authentication toward the proxy.
    pub fn authenticator_factory(mut self, factory: Option<AuthenticatorFactory>) -> Self {
        self.auth = factory;
        self
    }
    /// use the CONNECT method even for HTTP requests.
    pub fn always_use_connect(mut self, yesno: bool) -> Self {
        self.always_use_connect = yesno;
        self
    }
    /// use DIRECT when connecting the proxies failed
    pub fn direct_fallback(mut self, yesno: bool) -> Self {
        self.direct_fallback = yesno;
        self
    }
    /// Timeout to use when trying to estabish a new connection.
    pub fn connect_timeout(mut self, duration: Duration) -> Self {
        self.connect_timeout = Some(duration);
        self
    }

    pub fn build(self) -> Session {
        let pac_script = self
            .pac_script
            .unwrap_or_else(|| crate::DEFAULT_PAC_SCRIPT.into());
        let eval = Mutex::new(Evaluator::with_pac_script(&pac_script).unwrap());
        let auth = self.auth.unwrap_or(AuthenticatorFactory::None);
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
        Session(Arc::new(Shared {
            eval,
            direct_client: Mutex::new(Default::default()),
            proxy_clients: Default::default(),
            auth,
            always_use_connect: self.always_use_connect,
            direct_fallback: self.direct_fallback,
            connect_timeout: self.connect_timeout.unwrap_or(Duration::new(30, 0)),
            accesslog_tx,
        }))
    }
}
