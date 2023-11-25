pub mod accesslog;
pub mod context;
pub mod server;
pub mod session;
pub mod socket;

pub use crate::context::Context;
pub use crate::session::Session;

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
        detox_auth::netrc::Error,
    ),
    #[error("PAC script error: {0}")]
    PacScript(
        #[from]
        #[source]
        paclib::PacScriptError,
    ),
    // #[error("Invalid URI: {0}")]
    // InvalidURI(
    //     #[from]
    //     #[source]
    //     host_and_port::Error,
    // ),
}

pub(crate) mod body {
    use bytes::Bytes;
    use http_body_util::{combinators::BoxBody, BodyExt};

    pub(crate) fn empty() -> BoxBody<Bytes, hyper::Error> {
        http_body_util::Empty::new()
            .map_err(|never| match never {})
            .boxed()
    }

    pub(crate) fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
        http_body_util::Full::new(chunk.into())
            .map_err(|never| match never {})
            .boxed()
    }
}
