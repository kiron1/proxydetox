use std::fmt;

use detox_net::HostAndPort;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unknown directive {0}, expected DIRECT, PROXY, HTTP, or HTTPS")]
    UnknowDirective(String),
    #[error("empty entry ")]
    EmptyEntry,
    #[error("invalid input")]
    InvalidInput,
    #[error("endpoint parser error: {0}")]
    InvalidEndpoint(
        #[from]
        #[source]
        detox_net::host_and_port::Error,
    ),
}

/// Abstraction over the `FindProxyForUrl` return type.
///
/// See [Proxy Auto-Configuration (PAC) file, return value format](https://developer.mozilla.org/en-US/docs/Web/HTTP/Proxy_servers_and_tunneling/Proxy_Auto-Configuration_PAC_file#return_value_format)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Proxies(Vec<ProxyOrDirect>);

impl Proxies {
    pub fn new(proxies: Vec<ProxyOrDirect>) -> Self {
        Self(proxies)
    }

    pub fn direct() -> Self {
        Self::new(vec![ProxyOrDirect::Direct])
    }

    pub fn first(&self) -> ProxyOrDirect {
        self.0.first().unwrap().clone()
    }

    pub fn iter(&self) -> std::slice::Iter<ProxyOrDirect> {
        self.0.iter()
    }

    pub fn push(&mut self, p: ProxyOrDirect) {
        self.0.push(p)
    }
}

impl IntoIterator for Proxies {
    type Item = ProxyOrDirect;

    type IntoIter = std::vec::IntoIter<ProxyOrDirect>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for Proxies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, el) in self.0.iter().enumerate() {
            if i != 0 {
                f.write_str("; ")?;
            }
            el.fmt(f)?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Proxies {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result: Result<Vec<_>, _> = s
            .split(';')
            .map(|s| s.trim().trim_matches(';').trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse())
            .collect();
        match result {
            Ok(p) => {
                if p.is_empty() {
                    Err(Error::EmptyEntry)
                } else {
                    Ok(Proxies::new(p))
                }
            }
            Err(_) => Err(Error::InvalidInput),
        }
    }
}

/// Either a proxy or direct connection endpoint.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ProxyOrDirect {
    Direct,
    Proxy(Proxy),
}

/// A HTTP or HTTTPS proxy endpoint.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Proxy {
    Http(HostAndPort),
    Https(HostAndPort),
}

impl Proxy {
    pub fn endpoint(&self) -> &HostAndPort {
        match *self {
            Self::Http(ref ep) => ep,
            Self::Https(ref ep) => ep,
        }
    }

    pub fn host(&self) -> &str {
        self.endpoint().host()
    }

    pub fn port(&self) -> u16 {
        self.endpoint().port()
    }
}

impl fmt::Display for ProxyOrDirect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Direct => f.write_str("DIRECT"),
            Self::Proxy(ref proxy) => proxy.fmt(f),
        }
    }
}

impl fmt::Display for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Http(ref endpoint) => {
                f.write_str("HTTP ")?;
                f.write_str(&endpoint.to_string())
            }
            Self::Https(ref endpoint) => {
                f.write_str("HTTPS ")?;
                f.write_str(&endpoint.to_string())
            }
        }
    }
}

impl std::str::FromStr for ProxyOrDirect {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(host_port) = s.strip_prefix("HTTPS") {
            Ok(Self::Proxy(Proxy::Https(host_port.parse()?)))
        } else if let Some(host_port) = s.strip_prefix("PROXY") {
            Ok(Self::Proxy(Proxy::Http(host_port.parse()?)))
        } else if let Some(host_port) = s.strip_prefix("HTTP") {
            Ok(Self::Proxy(Proxy::Http(host_port.parse()?)))
        } else if s == "DIRECT" {
            Ok(Self::Direct)
        } else {
            Err(Error::UnknowDirective(s.to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Proxies;
    use super::Proxy;
    use super::ProxyOrDirect;

    #[test]
    fn proxy_parse() -> Result<(), Box<dyn std::error::Error>> {
        assert!("FOOBAR".parse::<ProxyOrDirect>().is_err());
        assert!("DIRECTx".parse::<ProxyOrDirect>().is_err());
        assert!("direct".parse::<ProxyOrDirect>().is_err());
        assert_eq!("DIRECT".parse::<ProxyOrDirect>()?, ProxyOrDirect::Direct);
        assert_eq!(" DIRECT ".parse::<ProxyOrDirect>()?, ProxyOrDirect::Direct);
        assert_eq!(
            "PROXY 127.0.0.1:3128".parse::<ProxyOrDirect>()?,
            ProxyOrDirect::Proxy(Proxy::Http("127.0.0.1:3128".parse()?))
        );
        assert_eq!(
            "HTTP 127.0.0.1:3128".parse::<ProxyOrDirect>()?,
            ProxyOrDirect::Proxy(Proxy::Http("127.0.0.1:3128".parse()?))
        );
        assert_eq!(
            "HTTPS 127.0.0.1:3128".parse::<ProxyOrDirect>()?,
            ProxyOrDirect::Proxy(Proxy::Https("127.0.0.1:3128".parse()?))
        );
        assert!("PROXY 127.0.0.1:abc".parse::<ProxyOrDirect>().is_err());
        Ok(())
    }

    #[test]
    fn proxies_parse() -> Result<(), Box<dyn std::error::Error>> {
        assert!("".parse::<Proxies>().is_err());
        assert!("FOO;BAR".parse::<Proxies>().is_err());
        assert!(";".parse::<Proxies>().is_err());
        assert_eq!(
            "DIRECT".parse::<Proxies>()?,
            Proxies::new(vec![ProxyOrDirect::Direct])
        );
        assert_eq!(
            "DIRECT;".parse::<Proxies>()?,
            Proxies::new(vec![ProxyOrDirect::Direct])
        );
        assert_eq!(
            "PROXY localhost:3128; DIRECT".parse::<Proxies>()?,
            Proxies::new(vec![
                ProxyOrDirect::Proxy(Proxy::Http("localhost:3128".parse()?)),
                ProxyOrDirect::Direct
            ])
        );
        assert_eq!(
            "HTTP localhost:3128; DIRECT".parse::<Proxies>()?,
            Proxies::new(vec![
                ProxyOrDirect::Proxy(Proxy::Http("localhost:3128".parse()?)),
                ProxyOrDirect::Direct
            ])
        );
        assert_eq!(
            "HTTPS localhost:3128; DIRECT".parse::<Proxies>()?,
            Proxies::new(vec![
                ProxyOrDirect::Proxy(Proxy::Https("localhost:3128".parse()?)),
                ProxyOrDirect::Direct
            ])
        );
        Ok(())
    }
}
