use std::{collections::HashMap, time::UNIX_EPOCH};

use boa_engine::class::Class;
use boa_gc::{Finalize, Trace};
use tracing::instrument;

pub type DnsMap = HashMap<String, (Option<String>, u64)>;

#[derive(Default, Debug, Trace, Finalize)]
pub struct DnsCache {
    map: DnsMap,
    cleanup_ttl: u64,
}

impl DnsCache {
    #[instrument(skip(self))]
    pub fn lookup(&mut self, host: &str) -> Option<String> {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let ttl = now + std::time::Duration::from_secs(5 * 60).as_secs();

        let resolve_and_insert = |map: &mut DnsMap, host: &str| -> Option<String> {
            let addr = resolve(host);
            map.insert(host.into(), (addr.clone(), ttl));
            addr
        };

        let result = if let Some(result) = self.map.get(host) {
            if result.1 < now {
                let addr = resolve_and_insert(&mut self.map, host);
                tracing::trace!(?addr, "expired");
                addr
            } else {
                let addr = result.0.clone();
                tracing::trace!(?addr, "hit");
                addr
            }
        } else {
            let addr = resolve_and_insert(&mut self.map, host);
            tracing::trace!(?addr, "miss");
            addr
        };

        if self.cleanup_ttl < now {
            self.cleanup(now);
            self.cleanup_ttl = ttl;
        };

        result
    }

    fn cleanup(&mut self, now: u64) {
        self.map.retain(|_, v| v.1 > now);
    }

    pub fn map(&self) -> DnsMap {
        self.map.clone()
    }
}

impl Class for DnsCache {
    const NAME: &'static str = "_DnsCache";

    fn constructor(
        _this: &boa_engine::JsValue,
        _args: &[boa_engine::JsValue],
        _context: &mut boa_engine::Context,
    ) -> boa_engine::JsResult<Self> {
        Ok(Default::default())
    }

    fn init(_class: &mut boa_engine::class::ClassBuilder) -> boa_engine::JsResult<()> {
        Ok(())
    }
}

// Resolve the host name and return the IP address as string, if resolvable.
pub(crate) fn resolve(host: &str) -> Option<String> {
    use std::net::ToSocketAddrs;

    let host_port = (host, 0u16);
    if let Ok(mut addrs) = host_port.to_socket_addrs() {
        if let Some(addr) = addrs.next() {
            let ip = addr.ip();
            Some(ip.to_string())
        } else {
            None
        }
    } else {
        None
    }
}
