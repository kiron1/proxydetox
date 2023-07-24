#[cfg(feature = "negotiate")]
pub mod negotiate;
pub mod netrc;

#[cfg(feature = "negotiate")]
use self::negotiate::NegotiateAuthenticator;
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
    Negotiate(Vec<String>),
}

impl AuthenticatorFactory {
    pub fn basic(store: netrc::Store) -> Self {
        AuthenticatorFactory::Basic(store)
    }

    #[cfg(feature = "negotiate")]
    pub fn negotiate(hosts: Vec<String>) -> Self {
        AuthenticatorFactory::Negotiate(hosts)
    }

    pub fn make(&self, proxy_fqdn: &str) -> Result<Box<dyn Authenticator>> {
        match self {
            Self::None => Ok(Box::new(NoneAuthenticator)),
            Self::Basic(ref store) => {
                let token = store.get(proxy_fqdn)?;
                Ok(Box::new(BasicAuthenticator::new(token)))
            }
            #[cfg(feature = "negotiate")]
            Self::Negotiate(ref hosts) => {
                if hosts.is_empty() || hosts.iter().any(|k| k == proxy_fqdn) {
                    // if the lists of hosts is empty, negotiate with all hosts
                    // otherwise only use negotiate for hosts in the allow list
                    Ok(Box::new(NegotiateAuthenticator::new(proxy_fqdn)?))
                } else {
                    // hosts which are not in the allow list will use no authentication
                    Ok(Box::new(NoneAuthenticator))
                }
            }
        }
    }
}

impl std::fmt::Display for AuthenticatorFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match *self {
            Self::None => f.write_str("none")?,
            Self::Basic(ref store) => {
                let hosts = store.hosts();
                if hosts.is_empty() {
                    f.write_str("basic")?;
                } else {
                    write!(f, "basic {}", store.hosts().join(","))?;
                }
            }
            #[cfg(feature = "negotiate")]
            Self::Negotiate(ref hosts) => {
                if hosts.is_empty() {
                    f.write_str("negotiate any")?;
                } else {
                    write!(f, "negotiate {}", hosts.join(","))?;
                }
            }
        };
        Ok(())
    }
}
