use quick_js::{Callback, JsValue};

#[derive(Debug)]
pub struct Resolver;

impl Callback<()> for Resolver {
    fn argument_count(&self) -> usize {
        1
    }

    fn call(&self, args: Vec<JsValue>) -> Result<Result<JsValue, String>, quick_js::ValueError> {
        let resolved = if let Some(host) = args[0].as_str() {
            resolve(host)
        } else {
            None
        };
        Ok(Ok(JsValue::from(resolved)))
    }
}

// Resolve the host name and return the IP address as string, if resolvable.
fn resolve(host: &str) -> Option<String> {
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
