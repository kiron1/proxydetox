use base64::Engine;
use http::{header::PROXY_AUTHORIZATION, HeaderValue};
use std::collections::HashMap;
use std::io::BufRead;
use std::sync::Arc;
use std::sync::RwLock;
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

    pub(crate) async fn step(
        &self,
        _last_headers: Option<hyper::HeaderMap>,
    ) -> crate::Result<hyper::HeaderMap> {
        let mut headers = hyper::HeaderMap::new();
        headers.append(
            PROXY_AUTHORIZATION,
            HeaderValue::from_str(&self.token).expect("valid header value"),
        );

        Ok(headers)
    }
}

#[derive(Default)]
struct Entries {
    /// Maps host names to base64 encoded login strings.
    hosts: HashMap<String, String>,
    /// Default entry of .netrc file
    default: Option<String>,
}

#[derive(Clone, Default)]
pub struct Store {
    entries: Arc<RwLock<Entries>>,
}

impl Store {
    pub fn new(input: impl BufRead) -> Result<Self, Error> {
        let entries = Self::map_from_netrc(input)?;
        let entries = Arc::new(RwLock::new(entries));
        Ok(Self { entries })
    }

    fn map_from_netrc(input: impl BufRead) -> Result<Entries, Error> {
        // Generate the `Basic base64("login:password") token
        fn make_token(login: &str, password: &str) -> String {
            let t = format!("{login}:{password}");
            format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD.encode(t)
            )
        }

        let netrc = netrc::Netrc::parse(input).map_err(|_| Error::NetrcParserError)?;

        let hosts = netrc
            .hosts
            .iter()
            .map(|(host, machine)| {
                (
                    host.to_owned(),
                    make_token(&machine.login, machine.password.as_deref().unwrap_or("")),
                )
            })
            .collect();
        let default = netrc
            .default
            .map(|m| make_token(&m.login, m.password.as_deref().unwrap_or("")));
        Ok(Entries { hosts, default })
    }

    pub fn update(&self, input: impl BufRead) -> Result<(), Error> {
        let mut entries = self.entries.write().unwrap();
        *entries = Self::map_from_netrc(input)?;
        Ok(())
    }

    pub fn hosts(&self) -> Vec<String> {
        self.entries.read().unwrap().hosts.keys().cloned().collect()
    }

    pub(crate) fn get(&self, k: &str) -> Result<String, Error> {
        let entries = self.entries.read().unwrap();
        let default = &entries.default;
        entries
            .hosts
            .get(k)
            .map(|k| k.to_owned())
            .or_else(|| default.clone())
            .ok_or_else(|| Error::NoEntryForHost(k.to_string()))
    }
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.entries.read().unwrap().hosts.keys())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::Store;

    #[test]
    fn store_new_get_update() {
        let netrc = Store::new(std::io::Cursor::new(
            "machine example.org\nlogin hello\npassword world\n",
        ))
        .unwrap();
        let e = netrc.get("example.org").unwrap();
        assert_eq!(e, "Basic aGVsbG86d29ybGQ=");
        netrc
            .update(std::io::Cursor::new(
                "machine example.net\nlogin Hello\npassword World\n",
            ))
            .unwrap();
        assert!(netrc.get("example.org").is_err());
        let e = netrc.get("example.net").unwrap();
        assert_eq!(e, "Basic SGVsbG86V29ybGQ=");
    }
}
