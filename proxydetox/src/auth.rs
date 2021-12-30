#[cfg(feature = "negotiate")]
pub mod kerberos;
pub mod netrc;

use std::path::PathBuf;

#[cfg(feature = "negotiate")]
use self::kerberos::NegotiateAuthenticator;
use self::netrc::BasicAuthenticator;
use futures::future;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;

pub trait Authenticator: Send + Sync {
    fn step<'async_trait>(
        &'async_trait self,
        last_headers: Option<hyper::HeaderMap>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<hyper::HeaderMap>> + Send + 'async_trait>,
    >;
}

struct NoneAuthenticator;

impl Authenticator for NoneAuthenticator {
    fn step<'async_trait>(
        &'async_trait self,
        _last_headers: Option<hyper::HeaderMap>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<hyper::HeaderMap>> + Send + 'async_trait>,
    > {
        Box::pin(future::ok(Default::default()))
    }
}

#[derive(Clone, Debug)]
pub enum AuthenticatorFactory {
    None,
    Basic(PathBuf),
    #[cfg(feature = "negotiate")]
    Negotiate,
}

impl AuthenticatorFactory {
    pub fn basic(netrc_path: PathBuf) -> Self {
        AuthenticatorFactory::Basic(netrc_path)
    }

    #[cfg(feature = "negotiate")]
    pub fn negotiate() -> Self {
        AuthenticatorFactory::Negotiate
    }

    pub fn make(&self, proxy_url: &http::Uri) -> Result<Box<dyn Authenticator>> {
        let proxy_fqdn = proxy_url.host().unwrap_or_default();
        match self {
            Self::None => Ok(Box::new(NoneAuthenticator)),
            Self::Basic(ref netrc_path) => Ok(Box::new(BasicAuthenticator::new(
                netrc_path.as_path(),
                proxy_fqdn,
            )?)),
            #[cfg(feature = "negotiate")]
            Self::Negotiate => Ok(Box::new(NegotiateAuthenticator::new(proxy_fqdn)?)),
        }
    }
}

impl std::fmt::Display for AuthenticatorFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let name = match *self {
            Self::None => "none",
            Self::Basic(ref _netrc_path) => "basic",
            #[cfg(feature = "negotiate")]
            Self::Negotiate => "negotiate",
        };
        f.write_str(name)
    }
}
