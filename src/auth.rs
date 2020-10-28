use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
pub enum Error {
    NoHomeEnv,
    NoNetrcFile,
    NetrcParserError,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoHomeEnv => write!(f, "HOME not set"),
            Self::NoNetrcFile => write!(f, "no ~/.netrc file"),
            Self::NetrcParserError => write!(f, "failed to parse ~/.netrc file"),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Auth {
    login: String,
    password: Option<String>,
}

impl Auth {
    fn new(login: String, password: Option<String>) -> Auth {
        Auth { login, password }
    }

    pub fn as_basic(&self) -> String {
        let auth_str = if let Some(ref password) = self.password {
            format!("{}:{}", self.login, password)
        } else {
            format!("{}", self.login)
        };
        format!("Basic {}", base64::encode(&auth_str))
    }
}

#[derive(Debug)]
pub struct AuthStore {
    netrc: netrc::Netrc,
}

impl AuthStore {
    pub fn new() -> Result<Self> {
        let mut netrc_path = dirs::home_dir().ok_or(Error::NoHomeEnv)?;
        netrc_path.push(".netrc");
        let input = File::open(netrc_path.as_path()).map_err(|_| Error::NoNetrcFile)?;
        let netrc =
            netrc::Netrc::parse(BufReader::new(input)).map_err(|_| Error::NetrcParserError)?;
        Ok(Self { netrc })
    }

    pub fn find(&self, host: &str) -> Option<Auth> {
        if let Some(machine) = self.netrc.hosts.iter().find(|&x| x.0 == host) {
            let login = machine.1.login.clone();
            let password = machine.1.password.clone();
            Some(Auth::new(login, password))
        } else {
            None
        }
    }
}
