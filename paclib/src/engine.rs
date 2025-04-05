use boa_engine::{
    js_string, Context, JsNativeError, JsResult, JsString, JsValue, NativeFunction, Source,
};
use http::Uri;
use std::convert::Infallible;
use std::net::IpAddr;
use std::result::Result;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{field::debug, instrument};

use crate::dns::DnsCache;
use crate::{domain, Proxies};
use crate::{FindProxyError, PacScriptError};

const PAC_UTILS: &str = include_str!("pac_utils.js");

pub struct Engine {
    js: Context,
    my_ip_addr: Arc<Mutex<IpAddr>>,
}

impl Engine {
    fn mkjs(my_ip_addr: Arc<Mutex<IpAddr>>) -> Context {
        let mut js = Context::default();

        js.register_global_class::<DnsCache>().unwrap();
        js.register_global_class::<domain::Table>().unwrap();
        js.register_global_builtin_callable(
            js_string!("alert"),
            1,
            NativeFunction::from_fn_ptr(alert),
        )
        .expect("register_global_property");
        js.register_global_builtin_callable(
            js_string!("dnsResolve"),
            1,
            NativeFunction::from_fn_ptr(dns_resolve),
        )
        .expect("register_global_property");

        // # Safety
        // We do not capture any varaibles which would require tracing.
        unsafe {
            js.register_global_builtin_callable(
                js_string!("myIpAddress"),
                0,
                NativeFunction::from_closure({
                    let ip = my_ip_addr.clone();
                    move |this, args, ctx| my_ip_address(&ip, this, args, ctx)
                }),
            )
            .expect("register_global_property");
        }

        let dns_cache = js
            .eval(Source::from_bytes("new _DnsCache();"))
            .expect("new DnsCache()");
        js.register_global_property(
            js_string!("_dnsCache"),
            dns_cache,
            boa_engine::property::Attribute::all(),
        )
        .expect("register_global_property");

        js.eval(Source::from_bytes(PAC_UTILS))
            .expect("eval pac_utils.js");
        js.eval(Source::from_bytes(crate::DEFAULT_PAC_SCRIPT))
            .expect("eval default PAC script");
        js
    }

    pub fn new() -> Self {
        let my_ip_addr = Arc::new(Mutex::new(IpAddr::from(std::net::Ipv4Addr::new(
            127, 0, 0, 1,
        ))));
        let js = Self::mkjs(my_ip_addr.clone());

        Self { js, my_ip_addr }
    }

    pub fn with_pac_script(pac_script: &str) -> Result<Self, PacScriptError> {
        let mut new = Self::new();
        new.set_pac_script(Some(pac_script))?;
        Ok(new)
    }

    pub fn set_pac_script(&mut self, pac_script: Option<&str>) -> Result<(), PacScriptError> {
        self.js = Self::mkjs(self.my_ip_addr.clone());
        let pac_script = pac_script.unwrap_or(crate::DEFAULT_PAC_SCRIPT);
        self.js
            .eval(Source::from_bytes(pac_script))
            .map_err(|e| PacScriptError::InternalError(e.to_string()))?;
        Ok(())
    }

    pub fn set_my_ip_address(&mut self, addr: IpAddr) -> Result<(), Infallible> {
        if let Ok(mut ip) = self.my_ip_addr.lock() {
            *ip = addr;
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self), ret, fields(duration))]
    pub fn find_proxy(&mut self, uri: &Uri) -> Result<Proxies, FindProxyError> {
        let host = uri.host().ok_or(FindProxyError::NoHost)?;

        let start = Instant::now();
        let find_proxy_fn = self
            .js
            .global_object()
            .get(js_string!("FindProxyForURL"), &mut self.js)
            .map_err(|e| FindProxyError::InternalError(e.to_string()))?;
        let proxy = match find_proxy_fn {
            JsValue::Object(find_proxy_fn) => {
                let uri = JsValue::from(JsString::from(uri.to_string()));
                let host = JsValue::from(JsString::from(host));
                find_proxy_fn
                    .call(&JsValue::Null, &[uri, host], &mut self.js)
                    .map_err(|e| FindProxyError::InternalError(e.to_string()))
            }
            _ => Err(FindProxyError::FindProxyForURLMissing)?,
        };
        tracing::Span::current().record("duration", debug(&start.elapsed()));

        let proxy = proxy?;
        match &proxy {
            JsValue::String(proxies) => {
                let proxies = proxies
                    .to_std_string()
                    .map_err(|_| FindProxyError::EmptyResult)?;
                proxies
                    .parse::<Proxies>()
                    .map_err(|_| FindProxyError::InvalidResult(proxies))
            }
            _ => Err(FindProxyError::InvalidResultType(format!(
                "{:?}",
                proxy.get_type()
            ))),
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

fn alert(_this: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Proxy_servers_and_tunneling/Proxy_Auto-Configuration_PAC_file#alert
    // https://developer.mozilla.org/en-US/docs/Web/API/Window/alert
    if let Some(message) = args.first() {
        let message = message
            .as_string()
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();
        println!("{}", &message)
    } else {
        println!();
    }
    Ok(JsValue::undefined())
}

fn dns_resolve(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let global = context.global_object();
    let dns_cache = global
        .get(js_string!("_dnsCache"), context)
        .expect("_dnsCache");
    let dns_cache = dns_cache.as_object();
    let mut dns_cache = dns_cache
        .and_then(|obj| obj.try_borrow_mut().ok())
        .expect("mut DnsCache");
    let dns_cache = dns_cache
        .downcast_mut::<DnsCache>()
        .expect("downcast_mut<DnsCache>");

    let Some(host) = args.first() else {
        return Err(JsNativeError::typ()
            .with_message("first argument is missing")
            .into());
    };
    let Ok(host) = host.to_string(context) else {
        return Err(JsNativeError::typ()
            .with_message("first argument must be string")
            .into());
    };
    let resolved = dns_cache.lookup(&host.to_std_string_escaped());
    let value = match resolved {
        Some(ip) => JsValue::from(JsString::from(ip)),
        None => JsValue::null(),
    };

    Ok(value)
}

fn my_ip_address(
    ip: &Arc<Mutex<IpAddr>>,
    _this: &JsValue,
    _args: &[JsValue],
    _ctx: &mut Context,
) -> JsResult<JsValue> {
    let ip = ip
        .lock()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| String::from("127.0.0.1"));
    Ok(JsValue::from(JsString::from(ip)))
}

#[cfg(test)]
mod tests {
    use super::Engine;
    use super::Uri;
    use crate::Proxies;
    use crate::ProxyOrDirect;

    #[test]
    fn test_find_proxy() -> Result<(), Box<dyn std::error::Error>> {
        let mut eval = Engine::new();
        assert_eq!(
            eval.find_proxy(&"http://localhost/".parse::<Uri>().unwrap())?,
            Proxies::new(vec![ProxyOrDirect::Direct])
        );
        Ok(())
    }

    #[test]
    fn test_alert() -> Result<(), Box<dyn std::error::Error>> {
        let mut eval = Engine::with_pac_script(
            "function FindProxyForURL(url, host) { alert(\"alert\"); return \"DIRECT\"; }",
        )?;
        assert_eq!(
            eval.find_proxy(&"http://localhost/".parse::<Uri>().unwrap())?,
            Proxies::new(vec![ProxyOrDirect::Direct])
        );
        Ok(())
    }
}
