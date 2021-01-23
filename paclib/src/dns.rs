use crate::DNS_CACHE_NAME;
use duktape::{ContextRef, Stack};
use std::collections::HashMap;

pub type DnsMap = HashMap<String, (Option<String>, std::time::Instant)>;

#[derive(Debug)]
pub struct DnsCache {
    map: DnsMap,
    last_cleanup: std::time::Instant,
}

impl Default for DnsCache {
    fn default() -> Self {
        Self {
            map: Default::default(),
            last_cleanup: std::time::Instant::now(),
        }
    }
}

impl DnsCache {
    pub fn lookup(&mut self, host: &str) -> Option<String> {
        let now = std::time::Instant::now();
        let ttl = now + std::time::Duration::from_secs(15 * 60);

        let resolve_and_insert = |map: &mut DnsMap, host: &str| -> Option<String> {
            log::debug!("resolve_and_insert {:?}", host);

            let addr = resolve(host);
            map.insert(host.into(), (addr.clone(), ttl));
            addr
        };
        log::debug!("lookup {}", host);

        let result = if let Some(result) = self.map.get(host) {
            log::debug!("cache {:?}", result);

            if result.1 >= ttl {
                resolve_and_insert(&mut self.map, host)
            } else {
                result.0.clone()
            }
        } else {
            resolve_and_insert(&mut self.map, host)
        };

        if self.last_cleanup > ttl {
            self.cleanup(&now);
            self.last_cleanup = ttl;
        };

        result
    }

    fn cleanup(&mut self, now: &std::time::Instant) {
        self.map.retain(|_, v| v.1 > *now);
    }

    pub fn map(&self) -> DnsMap {
        self.map.clone()
    }
}

// Resolve the host name and return the IP address as string, if resolvable.
fn resolve(host: &str) -> Option<String> {
    use std::net::ToSocketAddrs;

    let host_port = (&host[..], 0u16);
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

pub unsafe extern "C" fn dns_resolve(ctx: *mut duktape_sys::duk_context) -> i32 {
    let mut ctx = ContextRef::from(ctx);
    ctx.require_stack(4);

    // resolve the host name using the dns cache
    let value = if ctx.get_global_string(DNS_CACHE_NAME).is_ok() {
        if let Ok(dns_cache) = ctx.pop_ptr::<DnsCache>() {
            if let Ok(host) = ctx.get_string(0) {
                let addr = dns_cache.as_mut().and_then(|d| d.lookup(&host));
                log::debug!("resolved {} to {:?}", host, addr);
                addr
            } else {
                log::error!("failed to convert host to string");
                None
            }
        } else {
            log::error!("failed to get DnsCache");
            None
        }
    } else {
        log::error!("failed to get DnsCache ({}) from global", DNS_CACHE_NAME);
        None
    };

    if let Some(value) = value {
        if ctx.push_string(&value).is_err() {
            ctx.push_null();
        }
    } else {
        ctx.push_null();
    }

    // number return values from this JavaScript function (will be consumed from top of stack)
    1
}
