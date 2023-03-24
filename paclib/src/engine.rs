use boa_engine::{Context, JsResult, JsString, JsValue};
use http::Uri;
use std::result::Result;
use std::time::Instant;
use tracing::{field::debug, instrument};

use crate::dns::DnsCache;
use crate::Proxies;
use crate::{FindProxyError, PacScriptError};

const PAC_UTILS: &str = include_str!("pac_utils.js");

pub struct Engine {
    js: Context,
}

impl Engine {
    pub fn new() -> Self {
        let mut js = Context::default();

        js.register_global_class::<DnsCache>().unwrap();
        js.register_global_builtin_function("alert", 1, alert);
        js.register_global_builtin_function("dnsResolve", 1, dns_resolve);
        js.register_global_builtin_function("myIpAddress", 0, my_ip_address);

        let dns_cache = js.eval("new _DnsCache();").expect("new DnsCache()");
        js.register_global_property(
            "_dnsCache",
            dns_cache,
            boa_engine::property::Attribute::all(),
        );

        js.eval(PAC_UTILS).expect("eval pac_utils.js");
        js.eval(crate::DEFAULT_PAC_SCRIPT)
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
        self.js.eval(pac_script).map_err(|_| PacScriptError)?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self), ret, fields(duration))]
    pub fn find_proxy(&mut self, uri: &Uri) -> Result<Proxies, FindProxyError> {
        let host = uri.host().ok_or(FindProxyError::NoHost)?;

        let start = Instant::now();
        let result = self.js.eval(&format!(
            "FindProxyForURL(\"{}\", \"{}\")",
            &uri.to_string(),
            host
        ));
        tracing::Span::current().record("duration", debug(&start.elapsed()));

        match &result {
            Ok(JsValue::String(proxies)) => proxies
                .as_str()
                .parse::<Proxies>()
                .map_err(|_| FindProxyError::InvalidResult),

            Ok(_value) => Err(FindProxyError::InvalidResult),
            Err(_cause) => Err(FindProxyError::InternalError),
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
    if let Some(message) = args.get(0) {
        println!("{}", &message.as_string().cloned().unwrap_or_default())
    } else {
        println!();
    }
    Ok(JsValue::undefined())
}

fn dns_resolve(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let global = ctx.global_object().clone();
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
            let resolved = dns_cache.lookup(host.as_str());
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
    let hostname = gethostname::gethostname();
    let ip = hostname
        .into_string()
        .map(|h| crate::dns::resolve(&h))
        .ok()
        .flatten()
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
