use crate::auth::{Error, Result};
use http::{header::PROXY_AUTHORIZATION, HeaderValue};
use libsspi::Context;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct NegotiateAuthenticator {
    cx: Arc<Mutex<Context>>,
}

impl NegotiateAuthenticator {
    pub fn new(proxy_url: &http::Uri) -> Result<Self> {
        let proxy_fqdn = proxy_url.host().expect("URI with host");
        let cx = Context::new(&proxy_fqdn);
        let cx = cx.map_err(|cause| Error::permanent(Box::new(cause)))?;
        let cx = Arc::new(Mutex::new(cx));
        Ok(Self { cx })
    }
}

impl super::Authenticator for NegotiateAuthenticator {
    // Call `step` `while request.status() == http::StatusCode::PROXY_AUTHENTICATION_REQUIRED {}`.
    fn step<'async_trait>(
        &'async_trait self,
        _last_headers: Option<hyper::HeaderMap>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<hyper::HeaderMap>> + Send + 'async_trait>,
    > {
        let fut = async move {
            let mut headers = hyper::HeaderMap::new();

            let token = self.cx.lock().await.step();
            let token = token.map_err(|cause| Error::temporary(Box::new(cause)))?;

            let b64token = base64::encode(&*token);
            tracing::debug!("auth sspi token: {}", &b64token);

            let auth_str = format!("Negotiate {}", b64token);
            headers.append(
                PROXY_AUTHORIZATION,
                HeaderValue::from_str(&auth_str).expect("valid header value"),
            );
            Ok(headers)
        };

        Box::pin(fut)
    }
}
