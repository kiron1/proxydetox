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
    #[error("DNS parser error: {0}")]
    DnsParser(
        #[from]
        #[source]
        dns_parser::Error,
    ),
}

pub type Result<T> = std::result::Result<T, Error>;
