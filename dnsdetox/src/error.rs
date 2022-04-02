#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
    #[error("Operation timed out")]
    Timeout(
        #[from]
        #[source]
        tokio::time::error::Elapsed,
    ),
    #[error("Unexpected HTTP status code: {0}")]
    UnexpectedHttpStatusCode(http::StatusCode),
    #[error("DNS parser error: {0}")]
    DnsParser(
        #[from]
        #[source]
        dns_parser::Error,
    ),
    #[error("HTTP error: {0}")]
    Http(
        #[from]
        #[source]
        http::Error,
    ),
    #[error("HTTP connection error: {0}")]
    HttpConnection(
        #[from]
        #[source]
        hyper::Error,
    ),
    #[error("HTTP CONNECT error: {0}")]
    HttpConnect(
        #[from]
        #[source]
        proxy_client::http_connect_connector::Error,
    ),
    #[error("internal error: {0}")]
    JoinError(
        #[from]
        #[source]
        tokio::task::JoinError,
    ),
}

pub type Result<T> = std::result::Result<T, Error>;
