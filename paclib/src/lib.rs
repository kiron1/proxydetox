pub mod dns;
pub mod engine;
pub mod evaluator;
pub mod proxy;

pub use crate::engine::Engine;
pub use crate::evaluator::Evaluator;
pub use crate::proxy::Proxies;
pub use crate::proxy::Proxy;
pub use crate::proxy::ProxyOrDirect;

const DEFAULT_PAC_SCRIPT: &str = "function FindProxyForURL(url, host) { return \"DIRECT\"; }";

#[derive(thiserror::Error, Debug)]
pub enum CreateEvaluatorError {
    #[error("failed to evaluate PAC: {0}")]
    EvalPacFile(
        #[from]
        #[source]
        PacScriptError,
    ),
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid PAC script")]
pub enum PacScriptError {
    #[error("internal error: {0}")]
    InternalError(String),
    #[error("I/O error: {0}")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
}

#[derive(thiserror::Error, Debug)]
pub enum FindProxyError {
    #[error("no host in URL")]
    NoHost,
    #[error("invalid result from PAC script")]
    InvalidResult,
    #[error("internal error when processing PAC script: {0}")]
    InternalError(String),
}
