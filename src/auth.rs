pub mod basic;
#[cfg(feature = "gssapi")]
pub mod negotiate;

use basic::NetrcAuthenticator;

#[cfg(feature = "gssapi")]
use negotiate::GssAuthenticator;

#[derive(Debug)]
pub enum Error {
    NoHomeEnv,
    NoNetrcFile,
    NetrcParserError,
    #[cfg(feature = "gssapi")]
    GssApiError(libgssapi::error::Error),
}

impl std::error::Error for Error {}

#[cfg(feature = "gssapi")]
impl From<libgssapi::error::Error> for Error {
    fn from(cause: libgssapi::error::Error) -> Self {
        Self::GssApiError(cause)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoHomeEnv => write!(f, "HOME not set"),
            Self::NoNetrcFile => write!(f, "no ~/.netrc file"),
            Self::NetrcParserError => write!(f, "failed to parse ~/.netrc file"),
            #[cfg(feature = "gssapi")]
            Self::GssApiError(ref cause) => write!(f, "gssapi error: {}", cause),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Authenticator {
    None,
    Netrc(NetrcAuthenticator),
    #[cfg(feature = "gssapi")]
    Gss(GssAuthenticator),
}

impl Authenticator {
    pub fn none() -> Self {
        Self::None
    }

    pub fn netrc_for(proxy_url: &http::Uri) -> Self {
        let netrc = NetrcAuthenticator::new(&proxy_url).expect("netrc");
        Self::Netrc(netrc)
    }

    #[cfg(feature = "gssapi")]
    pub fn gss_for(proxy_url: &http::Uri) -> Self {
        let gss = GssAuthenticator::new(&proxy_url).expect("gssapi");
        Self::Gss(gss)
    }

    pub async fn step(&self, response: Option<&http::Response<hyper::Body>>) -> hyper::HeaderMap {
        match self {
            Self::None => Default::default(),
            Self::Netrc(ref netrc) => netrc.step(response),
            #[cfg(feature = "gssapi")]
            Self::Gss(ref gss) => gss.step(response).await,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AuthenticatorFactory {
    None,
    Netrc,
    #[cfg(feature = "gssapi")]
    Gss,
}

impl AuthenticatorFactory {
    pub fn netrc() -> Self {
        AuthenticatorFactory::Netrc
    }

    #[cfg(feature = "gssapi")]
    pub fn gss() -> Self {
        AuthenticatorFactory::Gss
    }

    pub fn make(&self, proxy_url: &http::Uri) -> Authenticator {
        match self {
            Self::None => Authenticator::none(),
            Self::Netrc => Authenticator::netrc_for(&proxy_url),
            #[cfg(feature = "gssapi")]
            Self::Gss => Authenticator::gss_for(&proxy_url),
        }
    }
}
