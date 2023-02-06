use std::fmt;

use detox_net::HostAndPort;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unknown directive, expected DIRECT or PROXY")]
    UnknowDirective,
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
pub struct Proxies(Vec<ProxyDesc>);

impl Proxies {
    pub fn new(proxies: Vec<ProxyDesc>) -> Self {
        Self(proxies)
    }

    pub fn direct() -> Self {
        Self::new(vec![ProxyDesc::Direct])
    }

    pub fn first(&self) -> ProxyDesc {
        self.0.get(0).unwrap().clone()
    }

    pub fn iter(&self) -> std::slice::Iter<ProxyDesc> {
        self.0.iter()
    }

    pub fn push(&mut self, p: ProxyDesc) {
        self.0.push(p)
    }
}

impl fmt::Display for Proxies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for el in &self.0 {
            write!(f, "{el};")?;
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

/// A single proxy directive.
///
/// Either `DIRECT` or `PROXY host:port`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ProxyDesc {
    Direct,
    Proxy(HostAndPort),
}

impl fmt::Display for ProxyDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProxyDesc::Direct => f.write_str("DIRECT"),
            ProxyDesc::Proxy(ref host) => {
                f.write_str("PROXY ")?;
                f.write_str(&host.to_string())
            }
        }
    }
}

impl std::str::FromStr for ProxyDesc {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(host_port) = s.strip_prefix("PROXY") {
            Ok(ProxyDesc::Proxy(host_port.parse()?))
        } else if s == "DIRECT" {
            Ok(ProxyDesc::Direct)
        } else {
            Err(Error::UnknowDirective)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Proxies;
    use super::ProxyDesc;

    #[test]
    fn proxy_desc_parse() -> Result<(), Box<dyn std::error::Error>> {
        assert!("FOOBAR".parse::<ProxyDesc>().is_err());
        assert!("DIRECTx".parse::<ProxyDesc>().is_err());
        assert!("direct".parse::<ProxyDesc>().is_err());
        assert_eq!("DIRECT".parse::<ProxyDesc>()?, ProxyDesc::Direct);
        assert_eq!(" DIRECT ".parse::<ProxyDesc>()?, ProxyDesc::Direct);
        assert_eq!(
            "PROXY 127.0.0.1:3128".parse::<ProxyDesc>()?,
            ProxyDesc::Proxy("127.0.0.1:3128".parse()?)
        );
        assert!("PROXY 127.0.0.1:abc".parse::<ProxyDesc>().is_err());
        Ok(())
    }

    #[test]
    fn proxies_parse() -> Result<(), Box<dyn std::error::Error>> {
        assert!("".parse::<Proxies>().is_err());
        assert!("FOO;BAR".parse::<Proxies>().is_err());
        assert!(";".parse::<Proxies>().is_err());
        assert_eq!(
            "DIRECT".parse::<Proxies>()?,
            Proxies::new(vec![ProxyDesc::Direct])
        );
        assert_eq!(
            "DIRECT;".parse::<Proxies>()?,
            Proxies::new(vec![ProxyDesc::Direct])
        );
        assert_eq!(
            "PROXY localhost:3128; DIRECT".parse::<Proxies>()?,
            Proxies::new(vec![
                ProxyDesc::Proxy("localhost:3128".parse()?),
                ProxyDesc::Direct
            ])
        );
        Ok(())
    }
}
