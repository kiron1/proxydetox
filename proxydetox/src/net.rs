use http::header::LOCATION;
use http::{Response, StatusCode, Uri};
use hyper::body::Buf;
use hyper::Body;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use tokio::net::TcpStream;
use tracing_attributes::instrument;

#[instrument(level = "debug")]
pub async fn dial(uri: &http::Uri) -> tokio::io::Result<TcpStream> {
    match (uri.host(), uri.port_u16()) {
        (Some(host), Some(port)) => TcpStream::connect((host, port)).await,
        (_, _) => Err(tokio::io::Error::new(
            tokio::io::ErrorKind::AddrNotAvailable,
            "invalid URI",
        )),
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
    use super::HttpGetProgress;
    use http::header::LOCATION;
    use http::Response;
    use http::StatusCode;
    use hyper::Body;

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
        dbg!(&progress);
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
        dbg!(&progress);
        assert!(progress.is_err());
    }

    #[tokio::test]
    async fn dial_error() {
        assert!(
            super::dial(&http::Uri::builder().path_and_query("/").build().unwrap())
                .await
                .is_err()
        );
    }
}
