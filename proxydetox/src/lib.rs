pub mod auth;
pub mod client;
pub mod io;
pub mod net;
pub mod session;

pub use crate::net::http_file;
pub use crate::session::Session;
pub use hyper::Server;

pub const DEFAULT_PAC_SCRIPT: &str = "function FindProxyForURL(url, host) { return \"DIRECT\"; }";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
    #[error("hyper error: {0}")]
    Hyper(
        #[from]
        #[source]
        hyper::Error,
    ),
    #[error("netrc error: {0}")]
    Netrc(
        #[from]
        #[source]
        crate::auth::netrc::Error,
    ),
}
