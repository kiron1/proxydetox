use http::{header::PROXY_AUTHORIZATION, HeaderValue};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::result::Result;

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
    pub fn new(netrc_path: &Path, proxy_fqdn: &str) -> Result<Self, Error> {
        let netrc = BasicAuthenticator::read_netrc(netrc_path)?;

        if let Some(&(_, ref machine)) = netrc.hosts.iter().find(|&x| x.0 == proxy_fqdn) {
            let token = if let Some(ref password) = machine.password {
                format!("{}:{}", machine.login, password)
            } else {
                machine.login.to_string()
            };
            let token = format!("Basic {}", base64::encode(&token));
            tracing::debug!("auth netrc {}@{}: ", &machine.login, &proxy_fqdn);
            Ok(Self { token })
        } else {
            Err(Error::NoEntryForHost(proxy_fqdn.into()))
        }
    }

    fn read_netrc(netrc_path: &Path) -> Result<netrc::Netrc, Error> {
        let input = File::open(netrc_path).map_err(|_| Error::NoNetrcFile)?;
        let netrc =
            netrc::Netrc::parse(BufReader::new(input)).map_err(|_| Error::NetrcParserError)?;
        Ok(netrc)
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
