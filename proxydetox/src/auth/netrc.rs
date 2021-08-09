use futures::future;
use http::{header::PROXY_AUTHORIZATION, HeaderValue};
use std::fs::File;
use std::io::BufReader;
use std::result::Result;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HOME not set")]
    NoHomeEnv,
    #[error("no ~/.netrc file")]
    NoNetrcFile,
    #[error("failed to parse ~/.netrc file")]
    NetrcParserError,
    #[error("no entry for this host: {0}")]
    NoEntryForHost(String),
}

#[derive(Debug, Clone)]
pub struct BasicAuthenticator {
    token: String,
}

impl BasicAuthenticator {
    pub fn new(proxy_fqdn: &str) -> Result<Self, Error> {
        let netrc = BasicAuthenticator::home_netrc()?;

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

    fn home_netrc() -> Result<netrc::Netrc, Error> {
        let netrc_path = {
            let mut netrc_path = dirs::home_dir().ok_or(Error::NoHomeEnv)?;
            netrc_path.push(".netrc");
            netrc_path
        };
        let input = File::open(netrc_path.as_path()).map_err(|_| Error::NoNetrcFile)?;
        let netrc =
            netrc::Netrc::parse(BufReader::new(input)).map_err(|_| Error::NetrcParserError)?;
        Ok(netrc)
    }
}

impl super::Authenticator for BasicAuthenticator {
    fn step<'async_trait>(
        &'async_trait self,
        _last_headers: Option<hyper::HeaderMap>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = crate::auth::Result<hyper::HeaderMap>>
                + Send
                + 'async_trait,
        >,
    > {
        let mut headers = hyper::HeaderMap::new();
        headers.append(
            PROXY_AUTHORIZATION,
            HeaderValue::from_str(&self.token).expect("valid header value"),
        );

        Box::pin(future::ok(headers))
    }
}
