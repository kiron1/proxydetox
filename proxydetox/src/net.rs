use http::header::LOCATION;
use http::StatusCode;
use hyper::body::Buf;
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

/// We currently support only IETF RFC 2616, which requires absolute URIs in case of an redirect
pub async fn http_file(mut uri: http::Uri) -> std::io::Result<String> {
    let client = hyper::Client::new();
    let mut max_redirects = 10i32;
    let res = loop {
        let res = client
            .get(uri.clone())
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("GET {}: {}", &uri, e)))?;
        match res.status() {
            StatusCode::OK => {
                break res;
            }
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
                    let location = location.parse().map_err(|e| {
                        Error::new(
                            ErrorKind::Other,
                            format!("parsing URI '{}' failed: {}", &location, e),
                        )
                    })?;
                    // use location URI for a new try
                    uri = location;
                } else {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("redirect in {}, but location header is missing", &uri),
                    ));
                }
            }
            // consider any other status code as invalid
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("unexpected status code {} in GET {}", res.status(), &uri),
                ));
            }
        }

        max_redirects -= 1;
        if max_redirects <= 0 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("too many redirects in GET {}", &uri),
            ));
        }
    };

    let body = hyper::body::aggregate(res)
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "aggregate"))?;
    let mut buffer = String::new();
    body.reader().read_to_string(&mut buffer)?;
    Ok(buffer)
}
