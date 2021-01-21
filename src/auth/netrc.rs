use super::{Error, Result};
use http::{header::PROXY_AUTHORIZATION, HeaderValue};
use std::fs::File;
use std::io::BufReader;

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
                format!("{}", machine.login)
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
            let mut netrc_path = dirs::home_dir().ok_or(Error::NoHomeEnv)?;
            netrc_path.push(".netrc");
            netrc_path
        };
        let input = File::open(netrc_path.as_path()).map_err(|_| Error::NoNetrcFile)?;
        let netrc =
            netrc::Netrc::parse(BufReader::new(input)).map_err(|_| Error::NetrcParserError)?;
        Ok(netrc)
    }

    pub fn step(
        &self,
        _response: Option<&http::Response<hyper::Body>>,
    ) -> Result<hyper::HeaderMap> {
        let mut headers = hyper::HeaderMap::new();
        if let Some(ref token) = self.token {
            headers.append(
                PROXY_AUTHORIZATION,
                HeaderValue::from_str(&token).expect("valid header value"),
            );
        }
        Ok(headers)
    }
}
