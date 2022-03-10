use http::{header::PROXY_AUTHORIZATION, HeaderValue};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::io::BufRead;
use std::sync::Arc;
use Result;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HOME not set")]
    NoHomeEnv,
    #[error("no ~/.netrc file")]
    NoNetrcFile,
    #[error("failed to parse ~/.netrc file")]
    NetrcParserError,
    #[error("no entry for host: {0}")]
    NoEntryForHost(String),
}

#[derive(Debug, Clone)]
pub struct BasicAuthenticator {
    token: String,
}

impl BasicAuthenticator {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

impl super::Authenticator for BasicAuthenticator {
    fn step(
        &self,
        _last_headers: Option<hyper::HeaderMap>,
    ) -> crate::auth::Result<hyper::HeaderMap> {
        let mut headers = hyper::HeaderMap::new();
        headers.append(
            PROXY_AUTHORIZATION,
            HeaderValue::from_str(&self.token).expect("valid header value"),
        );

        Ok(headers)
    }
}

#[derive(Clone, Default)]
pub struct Store {
    /// Maps host names to base64 encoded login strings.
    hosts: Arc<RwLock<HashMap<String, String>>>,
}

impl Store {
    pub fn new(input: impl BufRead) -> Result<Self, Error> {
        let hosts = Self::map_from_netrc(input)?;

        Ok(Self {
            hosts: Arc::new(RwLock::new(hosts)),
        })
    }

    fn map_from_netrc(input: impl BufRead) -> Result<HashMap<String, String>, Error> {
        let netrc = netrc::Netrc::parse(input).map_err(|_| Error::NetrcParserError)?;

        let hosts = netrc
            .hosts
            .iter()
            .map(|(host, machine)| {
                (
                    host.to_owned(),
                    format!(
                        "{}:{}",
                        machine.login,
                        machine.password.to_owned().unwrap_or_default()
                    ),
                )
            })
            .map(|(host, login)| (host, base64::encode(&login)))
            .map(|(host, login)| (host, format!("Basic {}", login)))
            .collect();
        Ok(hosts)
    }

    pub fn update(&self, input: impl BufRead) -> Result<(), Error> {
        let mut hosts = self.hosts.write();
        *hosts = Self::map_from_netrc(input)?;
        Ok(())
    }

    pub(crate) fn get(&self, k: &str) -> Result<String, Error> {
        let hosts = self.hosts.read();
        hosts
            .get(k)
            .map(|k| k.to_owned())
            .ok_or_else(|| Error::NoEntryForHost(k.to_string()))
    }
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.hosts.read().keys()).finish()
    }
}
