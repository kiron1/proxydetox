use std::{collections::HashMap, net::SocketAddr, time::UNIX_EPOCH};

use boa_engine::{JsData, class::Class};
use boa_gc::{Finalize, Trace};
use tracing::instrument;

pub type DnsMap = HashMap<String, (Option<String>, u64)>;

#[derive(Default, Debug, Trace, Finalize, JsData)]
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

    fn data_constructor(
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

    select_address((host, 0u16).to_socket_addrs().ok()?.into_iter())
}

fn select_address(addresses: impl Iterator<Item = SocketAddr>) -> Option<String> {
    let mut ipv6 = None;
    for address in addresses {
        match address.ip() {
            std::net::IpAddr::V4(ip) => return Some(ip.to_string()),
            std::net::IpAddr::V6(ip) => {
                ipv6.get_or_insert_with(|| ip.to_string());
            }
        }
    }
    ipv6
}

#[cfg(test)]
mod tests {
    use super::select_address;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    #[test]
    fn select_address_prefers_ipv4() {
        let addresses = [
            SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), 0),
        ];

        assert_eq!(select_address(addresses.into_iter()), Some("192.0.2.1".into()));
    }

    #[test]
    fn select_address_falls_back_to_ipv6() {
        let addresses = [SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0)];

        assert_eq!(select_address(addresses.into_iter()), Some("::1".into()));
    }
}
