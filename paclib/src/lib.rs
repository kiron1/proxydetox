pub mod dns;
pub mod evaluator;
pub mod proxy;

pub use crate::dns::DnsCache;
pub use crate::evaluator::Evaluator;
pub use crate::proxy::Proxies;
pub use crate::proxy::ProxyDesc;

const DNS_RESOLVE_NAME: &str = "dnsResolve";
const DNS_CACHE_NAME: &str = "_dnsResolveCache";
const DEFAULT_PAC_SCRIPT: &str = "function FindProxyForURL(url, host) { return \"DIRECT\"; }";
