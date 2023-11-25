use bytes::{Buf, Bytes};
use detox_net::stream::MaybeTlsStream;
use http::header::{CONNECTION, CONTENT_LENGTH, HOST, LOCATION};
use http::{HeaderValue, Request, Response, StatusCode, Uri};
use http_body_util::{BodyExt, Empty};
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use std::io::{Error, Read};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

static MAX_SIZE: u64 = 8 * 1024 * 1024;

async fn http_get(
    uri: Uri,
    tls_config: Arc<rustls::ClientConfig>,
) -> std::io::Result<Response<hyper::body::Incoming>> {
    let http::uri::Parts {
        scheme,
        authority,
        path_and_query,
        ..
    } = uri.into_parts();
    let scheme = scheme.unwrap_or(http::uri::Scheme::HTTP);
    let host = authority
        .as_ref()
        .map(|a| a.host())
        .ok_or_else(|| Error::other("Invalid URI: host missing"))?;
    let stream = if scheme == http::uri::Scheme::HTTPS {
        let port = authority
            .as_ref()
            .and_then(|a| a.port().map(|p| p.as_u16()))
            .unwrap_or(443);
        let stream = TcpStream::connect((host, port)).await?;
        let domain = rustls::ServerName::try_from(host)
            .map_err(|e| Error::other(format!("Invalid domain name: {e}")))?;
        let tls = TlsConnector::from(tls_config);
        let stream = tls.connect(domain, stream).await?;
        MaybeTlsStream::from(stream)
    } else {
        // scheme == &http::uri::Scheme::HTTP
        let port = authority
            .as_ref()
            .and_then(|a| a.port().map(|p| p.as_u16()))
            .unwrap_or(80);

        let stream = TcpStream::connect((host, port)).await?;
        MaybeTlsStream::from(stream)
    };

    let path_and_query =
        path_and_query.unwrap_or_else(|| http::uri::PathAndQuery::from_static("/"));
    let uri = Uri::builder()
        .path_and_query(path_and_query)
        .build()
        .expect("URI");
    let request = Request::get(uri)
        .header(HOST, host)
        .header(CONNECTION, HeaderValue::from_static("close"))
        .body(Empty::<Bytes>::new())
        .map_err(|e| Error::other(format!("Invalid HTTP request: {e}")))?;

    let (mut request_sender, connection) = http1::handshake(TokioIo::new(stream))
        .await
        .map_err(|e| Error::other(format!("HTTP handshake error: {e}")))?;

    let (response, _connection) = tokio::join!(request_sender.send_request(request), connection);

    let response = response.map_err(|e| Error::other(format!("HTTP error: {e}")))?;
    Ok(response)
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
            HttpGetProgress::Complete(response) => {
                let (response, body) = response.into_parts();
                let size = response
                    .headers
                    .get(CONTENT_LENGTH)
                    .and_then(|h| h.to_str().ok().and_then(|s| s.parse::<u64>().ok()))
                    .ok_or_else(|| Error::other("Unknown size".to_owned()))?;
                if size > MAX_SIZE {
                    return Err(Error::other(format!("Size to large: {size} > {MAX_SIZE}")));
                }
                let body = body
                    .collect()
                    .await
                    .map_err(|e| Error::other(format!("Failed to receive body: {e}")))?
                    .aggregate();
                let mut data = Vec::new();
                body.reader().read_to_end(&mut data)?;
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
