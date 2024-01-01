use http::Uri;
use std::fmt::{Display, Write};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid URI without host and port")]
    InvalidUri,
    #[error("invalid format: {0}")]
    InvalidFormat(String),
    #[error("invalid port: {0}")]
    InvalidPort(
        #[from]
        #[source]
        std::num::ParseIntError,
    ),
    #[error("host missing")]
    NoHost,
    #[error("port missing")]
    NoPort,
}

/// A IP with port endpoint in the `host:port` form.
///
/// ```
/// # use detox_net::HostAndPort;
/// let endpoint = "example.org:8080".parse::<HostAndPort>().unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HostAndPort(String, u16);

impl HostAndPort {
    pub fn try_from_uri(uri: &Uri) -> std::result::Result<HostAndPort, Error> {
        let Some(host) = uri.host() else {
            return Err(Error::NoHost);
        };
        let port = uri.port_u16().ok_or(Error::NoPort).or_else(|_| {
            uri.scheme().ok_or(Error::NoPort).and_then(|s| {
                if *s == http::uri::Scheme::HTTP {
                    Ok(80u16)
                } else if *s == http::uri::Scheme::HTTPS {
                    Ok(443u16)
                } else {
                    Err(Error::InvalidUri)
                }
            })
        })?;

        Ok(HostAndPort(
            host.trim_matches(|c| c == '[' || c == ']').to_owned(),
            port,
        ))
    }
    pub fn host(&self) -> &str {
        &self.0
    }

    pub fn port(&self) -> u16 {
        self.1
    }

    pub fn to_pair(&self) -> (String, u16) {
        (self.0.clone(), self.1)
    }
}

impl Display for HostAndPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)?;
        f.write_char(':')?;
        write!(f, "{}", self.1)
    }
}

impl std::str::FromStr for HostAndPort {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut host_port = s.split(':');
        match (host_port.next(), host_port.next()) {
            (Some(host), Some(port)) => Ok(HostAndPort(host.trim().to_owned(), port.parse()?)),
            _ => Err(Self::Err::InvalidFormat(s.to_string())),
        }
    }
}

impl std::net::ToSocketAddrs for HostAndPort {
    type Iter = std::vec::IntoIter<std::net::SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (self.0.as_str(), self.1).to_socket_addrs()
    }
}

impl TryFrom<HostAndPort> for Uri {
    type Error = http::Error;

    fn try_from(value: HostAndPort) -> Result<Self, Self::Error> {
        Uri::builder().authority(value.to_string()).build()
    }
}

#[cfg(test)]
mod tests {
    use super::HostAndPort;

    #[test]
    fn host_and_port_test() -> Result<(), Box<dyn std::error::Error>> {
        let uri1: http::Uri = "example.org".parse()?;
        let uri2: http::Uri = "example.org:8080".parse()?;
        let uri3: http::Uri = "http://example.org:8080".parse()?;
        let uri4: http::Uri = "http://example.org".parse()?;
        let uri5: http::Uri = "https://example.org".parse()?;
        // No scheme and no port is an error
        assert!(HostAndPort::try_from_uri(&uri1).is_err());
        assert_eq!(
            HostAndPort::try_from_uri(&uri2)?.to_string(),
            "example.org:8080"
        );
        assert_eq!(
            HostAndPort::try_from_uri(&uri3)?.to_string(),
            "example.org:8080"
        );
        assert_eq!(
            HostAndPort::try_from_uri(&uri4)?.to_string(),
            "example.org:80"
        );
        assert_eq!(
            HostAndPort::try_from_uri(&uri5)?.to_string(),
            "example.org:443"
        );

        Ok(())
    }
}
