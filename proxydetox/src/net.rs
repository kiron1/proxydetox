use http::header::LOCATION;
use http::{Response, StatusCode, Uri};
use hyper::body::Buf;
use hyper::Body;
use std::fmt::Display;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};

#[derive(thiserror::Error, Debug)]
pub enum HostAndPortError {
    #[error("invalid URI without host and port")]
    InvalidUri,
    #[error("host missing")]
    NoHost,
    #[error("port missing")]
    NoPort,
}

#[derive(Debug, Clone)]
pub struct HostAndPort(String, u16);

impl HostAndPort {
    pub fn try_from_uri(uri: &Uri) -> std::result::Result<HostAndPort, HostAndPortError> {
        match (uri.scheme(), uri.host(), uri.port_u16()) {
            (Some(scheme), Some(host), None) => {
                let port = if *scheme == http::uri::Scheme::HTTP {
                    80u16
                } else if *scheme == http::uri::Scheme::HTTPS {
                    443u16
                } else {
                    return Err(HostAndPortError::NoPort);
                };
                Ok(HostAndPort(host.to_owned(), port))
            }
            (_, Some(host), Some(port)) => Ok(HostAndPort(host.to_owned(), port)),
            (_, _, _) => Err(HostAndPortError::InvalidUri),
        }
    }
}

impl Display for HostAndPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", &self.0, self.1)
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

pub async fn read_to_string(res: Response<Body>) -> std::io::Result<String> {
    let body = hyper::body::aggregate(res)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, format!("aggregate: {}", e)))?;
    let mut buffer = String::new();
    body.reader().read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// We currently support only IETF RFC 2616, which requires absolute URIs in case of an redirect
pub async fn http_file(mut uri: Uri) -> std::io::Result<String> {
    let client = hyper::Client::new();
    let mut max_redirects = 10i32;
    let content = loop {
        let res = client
            .get(uri.clone())
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("GET {}: {}", &uri, e)))?;

        let progress = HttpGetProgress::from_response(res)?;
        match progress {
            HttpGetProgress::Complete(res) => break read_to_string(res).await,
            HttpGetProgress::Redirect(location) => uri = location,
        }

        max_redirects -= 1;
        if max_redirects <= 0 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("too many redirects in GET {}", &uri),
            ));
        }
    };
    content
}

#[derive(Debug)]
enum HttpGetProgress {
    Redirect(Uri),
    Complete(Response<Body>),
}

impl HttpGetProgress {
    fn from_response(res: Response<Body>) -> std::io::Result<Self> {
        match res.status() {
            StatusCode::OK => Ok(Self::Complete(res)),
            // received a redirect, we need to follow the url of the `location` header
            StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT => {
                if let Some(location) = res.headers().get(LOCATION) {
                    // get `location` header value as string
                    let location = location.to_str().map_err(|e| {
                        Error::new(
                            ErrorKind::Other,
                            format!("location value is not a valid string: {}", e),
                        )
                    })?;
                    // turn `location` string into an `Uri` object
                    let location = location.parse::<Uri>().map_err(|e| {
                        Error::new(
                            ErrorKind::Other,
                            format!("parsing URI '{}' failed: {}", &location, e),
                        )
                    })?;
                    if location.authority().is_none() {
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!("URI '{}' is not absolute", &location),
                        ));
                    }
                    // use location URI for a new try
                    Ok(Self::Redirect(location))
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "redirect, but location header is missing",
                    ))
                }
            }
            // consider any other status code as invalid
            _ => Err(Error::new(
                ErrorKind::Other,
                format!("unexpected status code {} for GET request", res.status()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HostAndPort;
    use super::HttpGetProgress;
    use http::header::LOCATION;
    use http::Response;
    use http::StatusCode;
    use hyper::Body;

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

    #[test]
    fn http_get_progress_ok() {
        let res = Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap();
        let progress = HttpGetProgress::from_response(res).unwrap();
        assert!(matches!(progress, HttpGetProgress::Complete(_)));
    }

    #[test]
    fn http_get_progress_not_found() {
        let res = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap();
        let progress = HttpGetProgress::from_response(res);
        assert!(progress.is_err());
    }

    #[test]
    fn http_get_progress_redirect() {
        let location = "http://exmaple.org/next";
        let location_uri = location.parse::<http::Uri>().unwrap();

        for status in &[
            StatusCode::MOVED_PERMANENTLY,
            StatusCode::FOUND,
            StatusCode::TEMPORARY_REDIRECT,
            StatusCode::PERMANENT_REDIRECT,
        ] {
            let res = Response::builder()
                .status(status)
                .header(LOCATION, location)
                .body(Body::empty())
                .unwrap();
            let progress = HttpGetProgress::from_response(res).unwrap();
            assert!(matches!(progress, HttpGetProgress::Redirect(_)));
            if let HttpGetProgress::Redirect(uri) = progress {
                assert_eq!(uri, location_uri);
            }
        }
    }

    #[test]
    fn http_get_progress_missing_location() {
        let res = Response::builder()
            .status(StatusCode::PERMANENT_REDIRECT)
            .body(Body::empty())
            .unwrap();
        let progress = HttpGetProgress::from_response(res);
        assert!(progress.is_err());
    }

    #[test]
    fn http_get_progress_relative_location() {
        let res = Response::builder()
            .status(StatusCode::FOUND)
            .header(LOCATION, "/index.html")
            .body(Body::empty())
            .unwrap();
        let progress = HttpGetProgress::from_response(res);
        assert!(progress.is_err());
    }

    #[test]
    fn http_get_progress_invalid_location() {
        let res = Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "\\")
            .body(Body::empty())
            .unwrap();
        let progress = HttpGetProgress::from_response(res);
        assert!(progress.is_err());
    }
}
