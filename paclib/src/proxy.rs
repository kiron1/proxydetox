use http::Uri;
use std::fmt;

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum Error {
    #[error("parser error")]
    ParserError,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ProxyDesc {
    Direct,
    Proxy(Uri),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Proxies(Vec<ProxyDesc>);

impl ProxyDesc {
    pub fn parse(input: &str) -> Result<ProxyDesc, Error> {
        let input = input.trim();
        if let Some(uri) = input.strip_prefix("PROXY") {
            let uri = uri.trim();
            let uri = uri.parse::<Uri>().map_err(|_| Error::ParserError)?;
            Ok(ProxyDesc::Proxy(uri))
        } else if input == "DIRECT" {
            Ok(ProxyDesc::Direct)
        } else {
            Err(Error::ParserError)
        }
    }
}

impl Proxies {
    pub fn new(proxies: Vec<ProxyDesc>) -> Self {
        Self(proxies)
    }

    pub fn direct() -> Self {
        Self::new(vec![ProxyDesc::Direct])
    }

    pub fn parse(input: &str) -> Result<Proxies, Error> {
        let result: Result<Vec<_>, _> = input
            .split(';')
            .map(|s| s.trim().trim_matches(';').trim())
            .filter(|s| !s.is_empty())
            .map(|s| ProxyDesc::parse(s))
            .collect();
        match result {
            Ok(p) => {
                if p.is_empty() {
                    Err(Error::ParserError)
                } else {
                    Ok(Proxies::new(p))
                }
            }
            Err(_) => Err(Error::ParserError),
        }
    }

    pub fn first(&self) -> ProxyDesc {
        self.0.get(0).unwrap().clone()
    }
}

impl fmt::Display for ProxyDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProxyDesc::Direct => write!(f, "DIRECT"),
            ProxyDesc::Proxy(ref url) => write!(f, "PROXY {}", url),
        }
    }
}

impl fmt::Display for Proxies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for el in &self.0 {
            write!(f, "{};", el)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Proxies;
    use super::ProxyDesc;
    use super::Uri;

    #[test]
    fn proxy_desc_parse() -> Result<(), Box<dyn std::error::Error>> {
        assert!(ProxyDesc::parse("FOOBAR").is_err());
        assert!(ProxyDesc::parse("DIRECTx").is_err());
        assert!(ProxyDesc::parse("direct").is_err());
        assert_eq!(ProxyDesc::parse("DIRECT")?, ProxyDesc::Direct);
        assert_eq!(ProxyDesc::parse(" DIRECT ")?, ProxyDesc::Direct);
        assert_eq!(
            ProxyDesc::parse("PROXY http://127.0.0.1:3128")?,
            ProxyDesc::Proxy("http://127.0.0.1:3128/".parse::<Uri>().unwrap())
        );
        Ok(())
    }

    #[test]
    fn proxies_parse() -> Result<(), Box<dyn std::error::Error>> {
        assert!(Proxies::parse("").is_err());
        assert!(Proxies::parse("FOO;BAR").is_err());
        assert_eq!(
            Proxies::parse("DIRECT")?,
            Proxies::new(vec![ProxyDesc::Direct])
        );
        assert_eq!(
            Proxies::parse("DIRECT;")?,
            Proxies::new(vec![ProxyDesc::Direct])
        );
        assert_eq!(
            Proxies::parse("PROXY http://localhost:3128/; DIRECT")?,
            Proxies::new(vec![
                ProxyDesc::Proxy("http://localhost:3128/".parse::<Uri>().unwrap()),
                ProxyDesc::Direct
            ])
        );
        Ok(())
    }
}
