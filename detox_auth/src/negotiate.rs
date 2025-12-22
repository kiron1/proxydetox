use base64::Engine;
use http::{
    HeaderValue,
    header::{PROXY_AUTHENTICATE, PROXY_AUTHORIZATION},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create context: {0}")]
    ContextCreationFailed(Box<dyn std::error::Error + Send + Sync>),
    #[error("Failed to execute authorization step: {0}")]
    AuthorizationStepFailed(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, Clone)]
pub struct NegotiateAuthenticator {
    proxy_fqdn: String,
}

impl NegotiateAuthenticator {
    pub fn new(proxy_fqdn: &str) -> std::result::Result<Self, Error> {
        Ok(Self {
            proxy_fqdn: proxy_fqdn.to_owned(),
        })
    }

    pub(crate) async fn step(
        &self,
        _last_headers: Option<hyper::HeaderMap>,
    ) -> crate::Result<hyper::HeaderMap> {
        let mut headers = hyper::HeaderMap::new();
        // let challenge = last_headers.map(|h| server_token(&h)).flatten();
        // let challenge = challenge.as_deref();
        let proxy_fqdn = self.proxy_fqdn.clone();
        let headers: crate::Result<_> = tokio::task::spawn_blocking(move || {
            let client_ctx = spnego::Context::new("HTTP", &proxy_fqdn);

            match client_ctx {
                Ok(mut cx) => match cx.step(None) {
                    Ok(Some(token)) => {
                        let b64token = base64::engine::general_purpose::STANDARD.encode(&*token);
                        let auth_str = format!("Negotiate {b64token}");
                        headers.append(
                            PROXY_AUTHORIZATION,
                            HeaderValue::from_str(&auth_str).expect("valid header value"),
                        );
                        Ok(headers)
                    }
                    Ok(None) => Ok(headers),
                    Err(cause) => Err(cause.into()),
                },
                Err(cause) => Err(cause.into()),
            }
        })
        .await?;
        headers
    }
}

// Extract the server token from "Proxy-Authenticate: Negotiate <base64>" header value
#[allow(unused)]
fn server_token(last_headers: &hyper::HeaderMap) -> Option<Vec<u8>> {
    last_headers
        .get_all(PROXY_AUTHENTICATE)
        .iter()
        .map(HeaderValue::to_str)
        .filter_map(std::result::Result::ok)
        .map(|s| s.splitn(2, ' '))
        .map(|mut i| (i.next(), i.next()))
        .filter_map(|k| if Some("Negotiate") == k.0 { k.1 } else { None })
        .map(|x| base64::engine::general_purpose::STANDARD.decode(x))
        .find_map(std::result::Result::ok)
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
