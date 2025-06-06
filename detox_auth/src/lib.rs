#[cfg(feature = "negotiate")]
pub mod negotiate;
pub mod netrc;

#[cfg(feature = "negotiate")]
use self::negotiate::NegotiateAuthenticator;
use self::netrc::BasicAuthenticator;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Authenticator {
    None,
    Basic(BasicAuthenticator),
    #[cfg(feature = "negotiate")]
    Negotiate(NegotiateAuthenticator),
}

impl Authenticator {
    pub async fn step(&self, last_headers: Option<hyper::HeaderMap>) -> Result<hyper::HeaderMap> {
        match self {
            Self::None => Ok(Default::default()),
            Self::Basic(basic) => basic.step(last_headers).await,
            #[cfg(feature = "negotiate")]
            Self::Negotiate(spnego) => spnego.step(last_headers).await,
        }
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
    pub fn none() -> Self {
        Self::None
    }

    pub fn basic(store: netrc::Store) -> Self {
        Self::Basic(store)
    }

    #[cfg(feature = "negotiate")]
    pub fn negotiate(hosts: Vec<String>) -> Self {
        Self::Negotiate(hosts)
    }

    pub fn make(&self, proxy_fqdn: &str) -> Result<Authenticator> {
        match self {
            Self::None => Ok(Authenticator::None),
            Self::Basic(store) => {
                let token = store.get(proxy_fqdn)?;
                Ok(Authenticator::Basic(BasicAuthenticator::new(token)))
            }
            #[cfg(feature = "negotiate")]
            Self::Negotiate(hosts) => {
                if hosts.is_empty() || hosts.iter().any(|k| k == proxy_fqdn) {
                    // if the lists of hosts is empty, negotiate with all hosts
                    // otherwise only use negotiate for hosts in the allow list
                    Ok(Authenticator::Negotiate(NegotiateAuthenticator::new(
                        proxy_fqdn,
                    )?))
                } else {
                    // hosts which are not in the allow list will use no authentication
                    Ok(Authenticator::None)
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
