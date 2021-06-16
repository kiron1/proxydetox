use crate::auth::{Error, Result};
use http::{
    header::{PROXY_AUTHENTICATE, PROXY_AUTHORIZATION},
    HeaderValue,
};
use libgssapi::{
    context::{ClientCtx, CtxFlags},
    credential::{Cred, CredUsage},
    name::Name,
    oid::{OidSet, GSS_MECH_KRB5, GSS_NT_HOSTBASED_SERVICE},
};
use tokio::task;

#[derive(Debug, Clone)]
pub struct NegotiateAuthenticator {
    proxy_url: http::Uri,
}

impl NegotiateAuthenticator {
    pub fn new(proxy_url: &http::Uri) -> Result<Self> {
        Ok(Self {
            proxy_url: proxy_url.clone(),
        })
    }

    fn make_client(
        proxy_url: &http::Uri,
    ) -> std::result::Result<ClientCtx, libgssapi::error::Error> {
        let desired_mechs = {
            let mut s = OidSet::new().expect("OidSet::new");
            s.add(&GSS_MECH_KRB5).expect("GSS_MECH_KRB5");
            s
        };

        let service_name = format!("http@{}", proxy_url.host().expect("URL with host"));
        let service_name = service_name.as_bytes();

        let name = Name::new(service_name, Some(&GSS_NT_HOSTBASED_SERVICE))?;
        let name = name.canonicalize(Some(&GSS_MECH_KRB5))?;

        let client_cred = Cred::acquire(None, None, CredUsage::Initiate, Some(&desired_mechs))?;

        Ok(ClientCtx::new(
            client_cred,
            name,
            CtxFlags::GSS_C_MUTUAL_FLAG,
            Some(&GSS_MECH_KRB5),
        ))
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
}

impl super::Authenticator for NegotiateAuthenticator {
    // Call `step` `while request.status() == http::StatusCode::PROXY_AUTHENTICATION_REQUIRED {}`.
    fn step<'async_trait>(
        &'async_trait self,
        last_headers: Option<hyper::HeaderMap>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<hyper::HeaderMap>> + Send + 'async_trait>,
    > {
        let fut = async move {
            // todo: actually the client context should be persistent across calls to step.
            // but currently there is no way to know when the context is completed and a new one needs
            // to be created. therefor we always create a fresh one (which seems to work).
            let mut headers = hyper::HeaderMap::new();

            let server_tok = last_headers.map(|h| Self::server_token(&h)).flatten();

            // Get client token, and create new gss client context.
            let token = {
                let proxy_url = self.proxy_url.clone();
                task::spawn_blocking(move || {
                    let stepper = Self::make_client(&proxy_url)?;
                    let token = server_tok.as_ref().map(|b| &**b);
                    let token = stepper.step(token);
                    token
                })
                .await
                .expect("join")
            };

            match token {
                Ok(Some(token)) => {
                    let b64token = base64::encode(&*token);
                    tracing::debug!("auth gss token: {}", &b64token);

                    let auth_str = format!("Negotiate {}", b64token);
                    headers.append(
                        PROXY_AUTHORIZATION,
                        HeaderValue::from_str(&auth_str).expect("valid header value"),
                    );
                }
                Ok(None) => {
                    // finished with setting up the token, cannot re-use ClinetCtx
                }
                Err(err) => {
                    tracing::error!(
                        "gss step error for {}: {} ({:?})",
                        &self.proxy_url,
                        &err,
                        &err
                    );
                    return Err(Error::temporary(Box::new(err)));
                }
            }
            Ok(headers)
        };

        Box::pin(fut)
    }
}

#[cfg(test)]
mod tests {
    use super::NegotiateAuthenticator;
    use super::PROXY_AUTHENTICATE;

    #[test]
    fn server_token_test() -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = hyper::HeaderMap::new();
        headers.append(
            PROXY_AUTHENTICATE,
            http::HeaderValue::from_str("Negotiate SGVsbG8gV29ybGQh").expect("valid header value"),
        );

        assert_eq!(
            NegotiateAuthenticator::server_token(&headers),
            Some(b"Hello World!".to_vec())
        );

        Ok(())
    }
}
