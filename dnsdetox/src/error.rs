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
    #[error("HTTP connection/protocoll error: {0}")]
    Hyper(
        #[from]
        #[source]
        hyper::Error,
    ),
    #[error("HTTP connection error: {0}")]
    HttpConnection(
        #[from]
        #[source]
        detox_hyper::conn::Error,
    ),
    #[error("Invalid endpoint: {0}")]
    Endpoint(
        #[from]
        #[source]
        detox_net::host_and_port::Error,
    ),
    #[error("Internal error: {0}")]
    JoinError(
        #[from]
        #[source]
        tokio::task::JoinError,
    ),
}

pub type Result<T> = std::result::Result<T, Error>;
