pub mod basic;
pub mod negotiate;

use self::{basic::NetrcAuthenticator, negotiate::GssAuthenticator};

#[derive(Debug)]
pub enum Error {
    NoHomeEnv,
    NoNetrcFile,
    NetrcParserError,
    GssApiError(libgssapi::error::Error),
}

impl std::error::Error for Error {}

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
            Self::GssApiError(ref cause) => write!(f, "gssapi error: {}", cause),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Authenticator {
    None,
    Netrc(NetrcAuthenticator),
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

    pub fn gss_for(proxy_url: &http::Uri) -> Self {
        let gss = GssAuthenticator::new(&proxy_url).expect("netrc");
        Self::Gss(gss)
    }

    pub async fn step(&self, response: Option<&http::Response<hyper::Body>>) -> hyper::HeaderMap {
        match self {
            Self::None => Default::default(),
            Self::Netrc(ref netrc) => netrc.step(response),
            Self::Gss(ref gss) => gss.step(response).await,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AuthenticatorFactory {
    None,
    Netrc,
    Gss,
}

impl AuthenticatorFactory {
    pub fn netrc() -> Self {
        AuthenticatorFactory::Netrc
    }

    pub fn gss() -> Self {
        AuthenticatorFactory::Gss
    }

    pub fn make(&self, proxy_url: &http::Uri) -> Authenticator {
        match self {
            Self::None => Authenticator::none(),
            Self::Netrc => Authenticator::netrc_for(&proxy_url),
            Self::Gss => Authenticator::gss_for(&proxy_url),
        }
    }
}
