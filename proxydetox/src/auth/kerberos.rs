use http::{
    header::{PROXY_AUTHENTICATE, PROXY_AUTHORIZATION},
    HeaderValue,
};
use std::result::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "negotiate")]
use libnegotiate::Context;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create context: {0}")]
    ContextCreationFailed(Box<dyn std::error::Error + Send + Sync>),
    #[error("Failed to execute authorization step: {0}")]
    AuthorizationStepFailed(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, Clone)]
pub struct NegotiateAuthenticator {
    cx: Arc<Mutex<Context>>,
}

impl NegotiateAuthenticator {
    pub fn new(proxy_fqdn: &str) -> Result<Self, Error> {
        let cx = Context::new(proxy_fqdn).map_err(|e| Error::ContextCreationFailed(Box::new(e)))?;
        let cx = Arc::new(Mutex::new(cx));

        Ok(Self { cx })
    }
}

impl super::Authenticator for NegotiateAuthenticator {
    fn step<'async_trait>(
        &'async_trait self,
        last_headers: Option<hyper::HeaderMap>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = crate::auth::Result<hyper::HeaderMap>>
                + Send
                + 'async_trait,
        >,
    > {
        let this = self.clone();
        let resp = async move {
            #[allow(unused_mut)]
            let mut cx = this.cx.lock().await;
            let mut headers = hyper::HeaderMap::new();
            let challenge = last_headers.map(|h| server_token(&h)).flatten();
            let challenge = challenge.as_deref();
            let token = cx.step(challenge).map_err(Box::new);

            match token {
                Ok(Some(token)) => {
                    let b64token = base64::encode(&*token);
                    tracing::debug!("negotiate token: {}", &b64token);
                    let auth_str = format!("Negotiate {}", b64token);
                    headers.append(
                        PROXY_AUTHORIZATION,
                        HeaderValue::from_str(&auth_str).expect("valid header value"),
                    );
                }
                Ok(None) => {}
                Err(err) => {
                    tracing::error!(
                        "negotiate error for {}: {} ({:?})",
                        &cx.target_name(),
                        &err,
                        &err
                    );
                    return Err(err.into());
                }
            }
            Ok(headers)
        };
        Box::pin(resp)
    }
}

// Extract the server token from "Proxy-Authenticate: Negotiate <base64>" header value
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
        .filter_map(std::result::Result::ok)
        .next();

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
