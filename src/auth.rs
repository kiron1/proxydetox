use std::io::BufReader;
use std::sync::Mutex;
use std::{fs::File, sync::Arc};

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

#[derive(Debug, Clone)]
pub struct NetrcAuthenticator {
    token: Option<String>,
}

impl NetrcAuthenticator {
    pub fn new(proxy_url: &http::Uri) -> Result<Self> {
        let netrc = NetrcAuthenticator::home_netrc()?;
        let host = proxy_url.host().expect("URI with host");

        let token = if let Some(&(_, ref machine)) = netrc.hosts.iter().find(|&x| x.0 == host) {
            let token = if let Some(ref password) = machine.password {
                format!("{}:{}", machine.login, password)
            } else {
                format!("{}", machine.login)
            };
            let token = format!("Basic {}", base64::encode(&token));
            tracing::debug!("auth netrc {}@{}: ", &machine.login, &proxy_url);
            Some(token)
        } else {
            None
        };

        Ok(Self { token })
    }

    fn home_netrc() -> Result<netrc::Netrc> {
        let netrc_path = {
            let mut netrc_path = dirs::home_dir().ok_or(Error::NoHomeEnv)?;
            netrc_path.push(".netrc");
            netrc_path
        };
        let input = File::open(netrc_path.as_path()).map_err(|_| Error::NoNetrcFile)?;
        let netrc =
            netrc::Netrc::parse(BufReader::new(input)).map_err(|_| Error::NetrcParserError)?;
        Ok(netrc)
    }

    fn step(&self, _response: Option<&http::Response<hyper::Body>>) -> hyper::HeaderMap {
        let mut headers = hyper::HeaderMap::new();
        if let Some(ref token) = self.token {
            headers.append(
                PROXY_AUTHORIZATION,
                HeaderValue::from_str(&token).expect("valid header value"),
            );
        }
        headers
    }
}

#[derive(Debug, Clone)]
pub struct GssAuthenticator {
    proxy_url: http::Uri,
    client: Arc<Mutex<ClientCtx>>,
}

impl GssAuthenticator {
    fn new(proxy_url: &http::Uri) -> Result<Self> {
        let client = GssAuthenticator::make_client(proxy_url)?;
        Ok(Self {
            proxy_url: proxy_url.clone(),
            client: Arc::new(Mutex::new(client)),
        })
    }

    fn make_client(proxy_url: &http::Uri) -> Result<ClientCtx> {
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

    fn step(&self, response: Option<&http::Response<hyper::Body>>) -> hyper::HeaderMap {
        let mut headers = hyper::HeaderMap::new();

        //while request.status() == http::StatusCode::PROXY_AUTHENTICATION_REQUIRED {}
        let mut server_tok: Option<Vec<u8>> = None;

        if let Some(response) = response {
            // Extract the server token from "Proxy-Authenticate: Negotiate <base64>" header value
            for auth in response.headers().get_all(PROXY_AUTHENTICATE) {
                if let Ok(auth) = auth.to_str() {
                    let mut split = auth.splitn(2, ' ');
                    if let Some(method) = split.next() {
                        if method == "Negotiate" {
                            if let Some(token) = split.next() {
                                if let Ok(token) = base64::decode(token) {
                                    server_tok = Some(token);
                                }
                            }
                        }
                    }
                }
            }
        }

        let token = self
            .client
            .lock()
            .unwrap()
            .step(server_tok.as_ref().map(|b| &**b));
        dbg!(&libgssapi::context::SecurityContext::info(
            &*self.client.lock().unwrap()
        ));

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
            Err(ref err) => tracing::error!("gss step error: {}", err),
        }

        *self.client.lock().unwrap() =
            GssAuthenticator::make_client(&self.proxy_url).expect("make_client");

        headers
    }
}

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

    pub fn step(&self, response: Option<&http::Response<hyper::Body>>) -> hyper::HeaderMap {
        match self {
            Self::None => Default::default(),
            Self::Netrc(ref netrc) => netrc.step(response),
            Self::Gss(ref gss) => gss.step(response),
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
