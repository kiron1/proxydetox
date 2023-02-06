use http::{
    header::{PROXY_AUTHENTICATE, PROXY_AUTHORIZATION},
    HeaderValue,
};
use std::result::Result;

use cross_krb5::{ClientCtx, InitiateFlags};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create context: {0}")]
    ContextCreationFailed(Box<dyn std::error::Error + Send + Sync>),
    #[error("Failed to execute authorization step: {0}")]
    AuthorizationStepFailed(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, Clone)]
pub struct NegotiateAuthenticator {
    target_principal: String,
}

impl NegotiateAuthenticator {
    pub fn new(proxy_fqdn: &str) -> Result<Self, Error> {
        let target_principal = format!("HTTP/{proxy_fqdn}");
        Ok(Self { target_principal })
    }
}

impl super::Authenticator for NegotiateAuthenticator {
    fn step(
        &self,
        _last_headers: Option<hyper::HeaderMap>,
    ) -> crate::auth::Result<hyper::HeaderMap> {
        let mut headers = hyper::HeaderMap::new();
        // let challenge = last_headers.map(|h| server_token(&h)).flatten();
        // let challenge = challenge.as_deref();
        let client_ctx = ClientCtx::new(InitiateFlags::empty(), None, &self.target_principal, None);

        match client_ctx {
            Ok((_pending, token)) => {
                let b64token = base64::encode(&*token);
                let auth_str = format!("Negotiate {b64token}");
                headers.append(
                    PROXY_AUTHORIZATION,
                    HeaderValue::from_str(&auth_str).expect("valid header value"),
                );
            }
            Err(cause) => {
                return Err(cause.into());
            }
        }
        Ok(headers)
    }
}

// Extract the server token from "Proxy-Authenticate: Negotiate <base64>" header value
#[allow(unused)]
fn server_token(last_headers: &hyper::HeaderMap) -> Option<Vec<u8>> {
    let server_tok = last_headers
        .get_all(PROXY_AUTHENTICATE)
        .iter()
        .map(HeaderValue::to_str)
        .filter_map(std::result::Result::ok)
        .map(|s| s.splitn(2, ' '))
        .map(|mut i| (i.next(), i.next()))
        .filter_map(|k| if Some("Negotiate") == k.0 { k.1 } else { None })
        .map(base64::decode)
        .find_map(std::result::Result::ok);

    server_tok
}

#[cfg(test)]
mod tests {
    #[test]
    fn server_token_test() -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = hyper::HeaderMap::new();
        headers.append(
            http::header::PROXY_AUTHENTICATE,
            http::HeaderValue::from_str("Negotiate SGVsbG8gV29ybGQh").expect("valid header value"),
        );

        assert_eq!(
            super::server_token(&headers),
            Some(b"Hello World!".to_vec())
        );

        Ok(())
    }
}
