use crate::auth::Result;
use futures::future;
use http::{header::PROXY_AUTHORIZATION, HeaderValue};
use std::fs::File;
use std::io::BufReader;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HOME not set")]
    NoHomeEnv,
    #[error("no ~/.netrc file")]
    NoNetrcFile,
    #[error("failed to parse ~/.netrc file")]
    NetrcParserError,
}

#[derive(Debug, Clone)]
pub struct BasicAuthenticator {
    token: Option<String>,
}

impl BasicAuthenticator {
    pub fn new(proxy_url: &http::Uri) -> Result<Self> {
        let netrc = BasicAuthenticator::home_netrc()?;
        let host = proxy_url.host().expect("URI with host");

        let token = if let Some(&(_, ref machine)) = netrc.hosts.iter().find(|&x| x.0 == host) {
            let token = if let Some(ref password) = machine.password {
                format!("{}:{}", machine.login, password)
            } else {
                machine.login.to_string()
            };
            let token = format!("Basic {}", base64::encode(&token));
            tracing::debug!("auth netrc {}@{}: ", &machine.login, &proxy_url);
            Some(token)
        } else {
            None
        };

        Ok(Self { token })
    }

    fn home_netrc() -> Result<netrc::Netrc> {
        let netrc_path = {
            let mut netrc_path = dirs::home_dir()
                .ok_or_else(|| super::Error::temporary(Box::new(Error::NoHomeEnv)))?;
            netrc_path.push(".netrc");
            netrc_path
        };
        let input = File::open(netrc_path.as_path())
            .map_err(|_| super::Error::temporary(Box::new(Error::NoNetrcFile)))?;
        let netrc = netrc::Netrc::parse(BufReader::new(input))
            .map_err(|_| super::Error::temporary(Box::new(Error::NetrcParserError)))?;
        Ok(netrc)
    }
}

impl super::Authenticator for BasicAuthenticator {
    fn step<'async_trait>(
        &'async_trait self,
        _response: Option<hyper::HeaderMap>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<hyper::HeaderMap>> + Send + 'async_trait>,
    > {
        let mut headers = hyper::HeaderMap::new();
        if let Some(ref token) = self.token {
            headers.append(
                PROXY_AUTHORIZATION,
                HeaderValue::from_str(&token).expect("valid header value"),
            );
        }

        Box::pin(future::ok(headers))
    }
}
