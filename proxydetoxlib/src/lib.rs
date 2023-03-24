pub mod accesslog;
pub mod auth;
pub mod client;
pub mod connect;
pub mod net;
pub mod session;
pub mod socket;

pub use crate::net::http_file;
pub use crate::session::Session;
pub use hyper::Server;

pub const DEFAULT_PAC_SCRIPT: &str = "function FindProxyForURL(url, host) { return \"DIRECT\"; }";

lazy_static::lazy_static! {
    static ref VERSION: String = {
        if let Some(hash) = option_env!("PROXYDETOX_BUILD_GIT_HASH") {
            format!("{} ({})", env!("CARGO_PKG_VERSION"), hash)
        } else {
            env!("CARGO_PKG_VERSION").to_owned()
        }
    };

    pub static ref VERSION_STR: &'static str = &VERSION;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
    #[error("HTTP error: {0}")]
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

    #[error("PAC script error: {0}")]
    PacScript(
        #[from]
        #[source]
        paclib::PacScriptError,
    ),
}
