use boa_engine::{Context, JsResult, JsString, JsValue, NativeFunction, Source};
use http::Uri;
use std::result::Result;
use std::time::Instant;
use tracing::{field::debug, instrument};

use crate::dns::DnsCache;
use crate::Proxies;
use crate::{FindProxyError, PacScriptError};

const PAC_UTILS: &str = include_str!("pac_utils.js");

pub struct Engine<'a> {
    js: Context<'a>,
}

impl<'a> Engine<'a> {
    pub fn new() -> Self {
        let mut js = Context::default();

        js.register_global_class::<DnsCache>().unwrap();
        js.register_global_builtin_callable("alert", 1, NativeFunction::from_fn_ptr(alert))
            .expect("register_global_property");
        js.register_global_builtin_callable(
            "dnsResolve",
            1,
            NativeFunction::from_fn_ptr(dns_resolve),
        )
        .expect("register_global_property");
        js.register_global_builtin_callable(
            "myIpAddress",
            0,
            NativeFunction::from_fn_ptr(my_ip_address),
        )
        .expect("register_global_property");

        let dns_cache = js
            .eval(Source::from_bytes("new _DnsCache();"))
            .expect("new DnsCache()");
        js.register_global_property(
            "_dnsCache",
            dns_cache,
            boa_engine::property::Attribute::all(),
        )
        .expect("register_global_property");

        js.eval(Source::from_bytes(PAC_UTILS))
            .expect("eval pac_utils.js");
        js.eval(Source::from_bytes(crate::DEFAULT_PAC_SCRIPT))
            .expect("eval default PAC script");

        Self { js }
    }

    pub fn with_pac_script(pac_script: &str) -> Result<Self, PacScriptError> {
        let mut new = Self::new();
        new.set_pac_script(Some(pac_script))?;
        Ok(new)
    }

    pub fn set_pac_script(&mut self, pac_script: Option<&str>) -> Result<(), PacScriptError> {
        let pac_script = pac_script.unwrap_or(crate::DEFAULT_PAC_SCRIPT);
        self.js
            .eval(Source::from_bytes(pac_script))
            .map_err(|e| PacScriptError::InternalError(e.to_string()))?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self), ret, fields(duration))]
    pub fn find_proxy(&mut self, uri: &Uri) -> Result<Proxies, FindProxyError> {
        let host = uri.host().ok_or(FindProxyError::NoHost)?;

        let start = Instant::now();
        let find_proxy_fn = self
            .js
            .global_object()
            .get("FindProxyForURL", &mut self.js)
            .map_err(|e| FindProxyError::InternalError(e.to_string()))?;
        let proxy = match find_proxy_fn {
            JsValue::Object(find_proxy_fn) => {
                let uri = JsValue::from(uri.to_string());
                let host = JsValue::from(host);
                find_proxy_fn
                    .call(&JsValue::Null, &[uri, host], &mut self.js)
                    .map_err(|e| FindProxyError::InternalError(e.to_string()))
            }
            _ => Err(FindProxyError::InvalidResult)?,
        };
        tracing::Span::current().record("duration", debug(&start.elapsed()));
        let proxy = proxy?;

        match &proxy {
            JsValue::String(proxies) => proxies
                .to_std_string()
                .unwrap_or_default()
                .parse::<Proxies>()
                .map_err(|_| FindProxyError::InvalidResult),
            _ => Err(FindProxyError::InvalidResult),
        }
    }
}

impl<'a> Default for Engine<'a> {
    fn default() -> Self {
        Self::new()
    }
}

fn alert(_this: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Proxy_servers_and_tunneling/Proxy_Auto-Configuration_PAC_file#alert
    // https://developer.mozilla.org/en-US/docs/Web/API/Window/alert
    if let Some(message) = args.get(0) {
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

fn dns_resolve(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let global = ctx.global_object();
    let dns_cache = global.get("_dnsCache", ctx).expect("_dnsCache");
    let dns_cache = dns_cache.as_object();
    let mut dns_cache = dns_cache
        .and_then(|obj| obj.try_borrow_mut().ok())
        .expect("mut DnsCache");
    let dns_cache = dns_cache
        .downcast_mut::<DnsCache>()
        .expect("downcast_mut<DnsCache>");

    let value = if let Some(host) = args.get(0) {
        if let Ok(host) = host.to_string(ctx) {
            let resolved = dns_cache.lookup(&host.to_std_string_escaped());
            match resolved {
                Some(ip) => JsValue::from(JsString::from(ip)),
                None => JsValue::undefined(),
            }
        } else {
            JsValue::undefined()
        }
    } else {
        JsValue::undefined()
    };

    Ok(value)
}

fn my_ip_address(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let ip = default_net::get_default_interface()
        .ok()
        .and_then(|i| i.ipv4.get(0).map(|i| i.addr.to_string()))
        .unwrap_or_else(|| String::from("127.0.0.1"));
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
