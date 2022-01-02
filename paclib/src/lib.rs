pub mod dns;
pub mod evaluator;
pub mod proxy;

pub use crate::evaluator::Evaluator;
pub use crate::proxy::Proxies;
pub use crate::proxy::ProxyDesc;

pub use http::Uri;

const DNS_RESOLVE_NAME: &str = "dnsResolve";
