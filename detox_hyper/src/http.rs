use bytes::Bytes;
use detox_net::HostAndPort;
use http::header::{CONTENT_LENGTH, LOCATION};
use http::{Request, Response, StatusCode, Uri};
use http_body_util::{BodyExt, Empty};
use std::io::Error;
use std::sync::Arc;

use crate::conn::Connection;

static MAX_SIZE: u64 = 8 * 1024 * 1024;

async fn http_get(
    uri: Uri,
    tls_config: Arc<rustls::ClientConfig>,
) -> std::io::Result<Response<hyper::body::Incoming>> {
    let dst = HostAndPort::try_from_uri(&uri).map_err(std::io::Error::other)?;
    let scheme = uri.scheme().unwrap_or(&http::uri::Scheme::HTTP);
    let conn = if scheme == &http::uri::Scheme::HTTPS {
        Connection::https(dst, tls_config)
    } else {
        Connection::http(dst)
    };
    let conn = conn.await?;

    let request = Request::get(uri)
        .body(Empty::<Bytes>::new())
        .map_err(|e| Error::other(format!("Invalid HTTP request: {e}")))?;

    let conn = conn
        .handshake()
        .await
        .map_err(|e| Error::other(format!("HTTP handshake error: {e}")))?;

    conn.send_request(request)
        .await
        .map_err(|e| Error::other(format!("HTTP error: {e}")))
}

/// We currently support only IETF RFC 2616, which requires absolute URIs in case of an redirect
pub async fn http_file(
    mut uri: Uri,
    tls_config: Arc<rustls::ClientConfig>,
) -> std::io::Result<String> {
    let mut max_redirects = 10i32;

    loop {
        let response = http_get(uri.clone(), tls_config.clone()).await.map_err({
            let uri = uri.clone();
            move |e| Error::other(format!("GET {}: {}", &uri, e))
        })?;

        let progress = HttpGetProgress::from_response(response)?;
        match progress {
            HttpGetProgress::Complete(mut response) => {
                let size = response
                    .headers()
                    .get(CONTENT_LENGTH)
                    .and_then(|h| h.to_str().ok().and_then(|s| s.parse::<u64>().ok()));
                if let Some(size) = size {
                    if size > MAX_SIZE {
                        return Err(Error::other(format!("Size to large: {size} > {MAX_SIZE}")));
                    }
                }
                let mut data = Vec::new();

                while let Some(next) = response.frame().await {
                    let frame = next.map_err(|e| Error::other(format!("frameing error: {e}")))?;
                    if let Some(chunk) = frame.data_ref() {
                        data.extend_from_slice(chunk);
                        if data.len() > MAX_SIZE as usize {
                            return Err(Error::other(format!(
                                "Size to large: {} > {MAX_SIZE}",
                                data.len()
                            )));
                        }
                    }
                }

                let data = String::from_utf8(data)
                    .map_err(|e| Error::other(format!("Invalid UTF-8 data: {e}")))?;
                break Ok(data);
            }
            HttpGetProgress::Redirect(location) => uri = location,
        }

        max_redirects -= 1;
        if max_redirects <= 0 {
            return Err(Error::other(format!("too many redirects in GET {}", &uri)));
        }
    }
}

#[derive(Debug)]
enum HttpGetProgress<B> {
    Redirect(Uri),
    Complete(Response<B>),
}

impl<B> HttpGetProgress<B>
where
    B: hyper::body::Body,
{
    fn from_response(res: Response<B>) -> std::io::Result<Self>
    where
        B: hyper::body::Body,
    {
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
                        Error::other(format!("location value is not a valid string: {e}"))
                    })?;
                    // turn `location` string into an `Uri` object
                    let location = location.parse::<Uri>().map_err(|e| {
                        Error::other(format!("parsing URI '{}' failed: {}", &location, e))
                    })?;
                    if location.authority().is_none() {
                        return Err(Error::other(format!("URI '{}' is not absolute", &location)));
                    }
                    // use location URI for a new try
                    Ok(Self::Redirect(location))
                } else {
                    Err(Error::other("redirect, but location header is missing"))
                }
            }
            // consider any other status code as invalid
            _ => Err(Error::other(format!(
                "unexpected status code {} for GET request",
                res.status()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::header::LOCATION;
    use http::Response;
    use http::StatusCode;

    #[test]
    fn http_get_progress_ok() {
        let res = Response::builder()
            .status(StatusCode::OK)
            .body(Empty::<Bytes>::new())
            .unwrap();
        let progress = HttpGetProgress::from_response(res).unwrap();
        assert!(matches!(progress, HttpGetProgress::Complete(_)));
    }

    #[test]
    fn http_get_progress_not_found() {
        let res = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Empty::<Bytes>::new())
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
                .body(Empty::<Bytes>::new())
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
            .body(Empty::<Bytes>::new())
            .unwrap();
        let progress = HttpGetProgress::from_response(res);
        assert!(progress.is_err());
    }

    #[test]
    fn http_get_progress_relative_location() {
        let res = Response::builder()
            .status(StatusCode::FOUND)
            .header(LOCATION, "/index.html")
            .body(Empty::<Bytes>::new())
            .unwrap();
        let progress = HttpGetProgress::from_response(res);
        assert!(progress.is_err());
    }

    #[test]
    fn http_get_progress_invalid_location() {
        let res = Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "\\")
            .body(Empty::<Bytes>::new())
            .unwrap();
        let progress = HttpGetProgress::from_response(res);
        assert!(progress.is_err());
    }
}
