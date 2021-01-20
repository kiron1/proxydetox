#[cfg(feature = "gssapi")]
pub mod gssapi;
pub mod netrc;

use self::netrc::BasicAuthenticator;

#[cfg(feature = "gssapi")]
use gssapi::NegotiateAuthenticator;

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
    Basic(BasicAuthenticator),
    #[cfg(feature = "gssapi")]
    Negotiate(NegotiateAuthenticator),
}

impl Authenticator {
    pub fn none() -> Self {
        Self::None
    }

    pub fn basic_for(proxy_url: &http::Uri) -> Self {
        let basic = BasicAuthenticator::new(&proxy_url).expect("netrc");
        Self::Basic(basic)
    }

    #[cfg(feature = "gssapi")]
    pub fn negotiate_for(proxy_url: &http::Uri) -> Self {
        let negotiate = NegotiateAuthenticator::new(&proxy_url).expect("negotiate");
        Self::Negotiate(negotiate)
    }

    pub async fn step(&self, response: Option<&http::Response<hyper::Body>>) -> hyper::HeaderMap {
        match self {
            Self::None => Default::default(),
            Self::Basic(ref basic) => basic.step(response),
            #[cfg(feature = "gssapi")]
            Self::Negotiate(ref negotiate) => negotiate.step(response).await,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AuthenticatorFactory {
    None,
    Basic,
    #[cfg(feature = "gssapi")]
    Negotiate,
}

impl AuthenticatorFactory {
    pub fn basic() -> Self {
        AuthenticatorFactory::Basic
    }

    #[cfg(feature = "gssapi")]
    pub fn negotiate() -> Self {
        AuthenticatorFactory::Negotiate
    }

    pub fn make(&self, proxy_url: &http::Uri) -> Authenticator {
        match self {
            Self::None => Authenticator::none(),
            Self::Basic => Authenticator::basic_for(&proxy_url),
            #[cfg(feature = "gssapi")]
            Self::Negotiate => Authenticator::negotiate_for(&proxy_url),
        }
    }
}
