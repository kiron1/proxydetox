#[cfg(feature = "negotiate")]
pub mod kerberos;
pub mod netrc;

#[cfg(feature = "negotiate")]
use self::kerberos::NegotiateAuthenticator;
use self::netrc::BasicAuthenticator;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;

pub trait Authenticator: Send + Sync {
    fn step(&self, last_headers: Option<hyper::HeaderMap>) -> Result<hyper::HeaderMap>;
}

pub struct NoneAuthenticator;

impl Authenticator for NoneAuthenticator {
    fn step(&self, _last_headers: Option<hyper::HeaderMap>) -> Result<hyper::HeaderMap> {
        Ok(Default::default())
    }
}

#[derive(Clone, Debug)]
pub enum AuthenticatorFactory {
    None,
    Basic(netrc::Store),
    #[cfg(feature = "negotiate")]
    Negotiate,
}

impl AuthenticatorFactory {
    pub fn basic(store: netrc::Store) -> Self {
        AuthenticatorFactory::Basic(store)
    }

    #[cfg(feature = "negotiate")]
    pub fn negotiate() -> Self {
        AuthenticatorFactory::Negotiate
    }

    pub fn make(&self, proxy_fqdn: &str) -> Result<Box<dyn Authenticator>> {
        match self {
            Self::None => Ok(Box::new(NoneAuthenticator)),
            Self::Basic(ref store) => {
                let token = store.get(proxy_fqdn)?;
                Ok(Box::new(BasicAuthenticator::new(token)))
            }
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
