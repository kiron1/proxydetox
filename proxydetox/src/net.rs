use detox_net::Metered;
use http::header::LOCATION;
use http::{Response, StatusCode, Uri};
use hyper::body::Buf;
use hyper::Body;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::time::Instant;
use tokio::io::{AsyncRead, AsyncWrite};

/// Calls tokio::io::copy_bidirectional but ignores some of the common errors.
pub(crate) async fn copy_bidirectional<A, B>(
    upstream: &mut A,
    downstream: &mut B,
) -> Result<(), std::io::Error>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    let mut upstream = Metered::new(upstream);
    let mut downstream = Metered::new(downstream);
    let begin = Instant::now();
    let cp = tokio::io::copy_bidirectional(&mut upstream, &mut downstream)
        .await
        .map(|_| ());

    // Ignore errors which we cannot influence (e.g. peer is terminating the
    // connection without a clean shutdown/close)
    #[cfg(not(debug_assertions))]
    let cp = match cp {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            ErrorKind::ConnectionReset | ErrorKind::BrokenPipe => Ok(()),
            _ => Err(e),
        },
    };

    if let Err(ref cause) = cp {
        let dt = Instant::now() - begin;
        tracing::error!(
            %cause,
            ?dt,
            upstream_in = %upstream.bytes_read(),
            upstream_out = %upstream.bytes_written(),
            downstream_in = %downstream.bytes_read(),
            downstream_out = %downstream.bytes_written(),
            "tunnel error"
        );
    }
    cp
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

    loop {
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
    }
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
