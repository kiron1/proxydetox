use crate::Proxies;
use crate::Uri;
use duktape::{Context, ContextRef, Stack};
use std::result::Result;

const PAC_UTILS: &str = include_str!("pac_utils.js");

pub struct Evaluator {
    js: Context,
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

unsafe extern "C" fn dns_resolve(ctx: *mut duktape_sys::duk_context) -> i32 {
    let mut ctx = ContextRef::from(ctx);
    ctx.require_stack(4);

    ctx.push_global_stash();
    ctx.dup(0); // stack: [ host, stash, host ]

    // get already resolved host, or undefined
    // stack will be: [ host, stash, ip|undefined ]
    ctx.get_prop(1);

    if ctx.is_undefined(-1) {
        // remove the undefined from top of the stack
        ctx.drop(); // stack: [ host, stash ]
        assert_eq!(2, ctx.top());

        // we need to resolve the hostname for the first time
        let value = if let Ok(host) = ctx.get_string(0) {
            resolve(&host)
        } else {
            None
        };

        // stack will be: [ host, stash, ip ]
        if let Some(value) = value {
            if !ctx.push_string(&value).is_ok() {
                ctx.push_null();
            }
        } else {
            ctx.push_null();
        }

        // stack will be: [ host, stash, ip, host, ip ]
        ctx.dup(0);
        ctx.dup(2);
        assert_eq!(5, ctx.top());

        // store result in stash
        // before: [ host, stash, ip, host, ip ]
        // after: [ host, stash, ip ]
        ctx.put_prop(1);
        assert_eq!(3, ctx.top());
    } else {
        // already resolved, re-use it.
        // the current value on the stack is the result, we are done here.
        ctx.dup(-1);
    }

    // number return values from this JavaScript function (will be consumed from top of stack)
    1
}

impl Evaluator {
    pub fn new(pac_script: &str) -> Result<Self, CreateEvaluatorError> {
        let mut ctx = Context::new().map_err(|_| CreateEvaluatorError::CreateContext)?;
        ctx.push_c_function("dnsResolve", dns_resolve, 1)
            .map_err(|_| CreateEvaluatorError::CreateContext)?;
        ctx.eval(PAC_UTILS).expect("eval pac_utils.js");
        ctx.eval(pac_script)
            .map_err(|_| CreateEvaluatorError::EvalPacFile)?;
        Ok(Evaluator { js: ctx })
    }

    pub fn find_proxy(&mut self, uri: &Uri) -> Result<Proxies, FindProxyError> {
        let host = uri.host().ok_or(FindProxyError::NoHost)?;
        // FIXME: when something goes wrong here we need to clean up the stack!
        let result = {
            self.js
                .get_global_string("FindProxyForURL")
                .map_err(|_| FindProxyError::InternalError)?;
            self.js
                .push_string(&uri.to_string())
                .map_err(|_| FindProxyError::InternalError)?;
            self.js
                .push_string(host)
                .map_err(|_| FindProxyError::InternalError)?;
            self.js.call(2);
            self.js.pop()
        };

        match &result {
            Ok(duktape::Value::String(ref result)) => {
                Ok(Proxies::parse(result).map_err(|_| FindProxyError::InvalidResult)?)
            }
            _ => Err(FindProxyError::InvalidResult),
        }
    }

    #[cfg(test)]
    fn dns_resolve(&mut self, host: &str) -> Option<String> {
        // FIXME: when something goes wrong here we need to clean up the stack!
        let result = {
            self.js.get_global_string("dnsResolve").unwrap();
            self.js.push_string(host).unwrap();
            self.js.call(1);
            self.js.pop()
        };

        match result {
            Ok(duktape::Value::String(result)) => Some(result),
            Ok(duktape::Value::Null) => None,
            _ => panic!("invalid result type"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CreateEvaluatorError {
    CreateContext,
    EvalPacFile,
}

impl std::error::Error for CreateEvaluatorError {}

impl std::fmt::Display for CreateEvaluatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match *self {
            CreateEvaluatorError::CreateContext => write!(f, "failed to create JS context"),
            CreateEvaluatorError::EvalPacFile => write!(f, "failed to evaluate PAC"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FindProxyError {
    NoHost,
    InvalidResult,
    InternalError,
}

impl std::error::Error for FindProxyError {}

impl std::fmt::Display for FindProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match *self {
            FindProxyError::NoHost => write!(f, "no host in URL"),
            FindProxyError::InvalidResult => write!(f, "invalid result from PAC script"),
            FindProxyError::InternalError => write!(f, "internal error when processing PAC script"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Evaluator;
    use super::Uri;
    use crate::Proxies;
    use crate::ProxyDesc;

    const TEST_PAC_SCRIPT: &str = "function FindProxyForURL(url, host) { return \"DIRECT\"; }";

    #[test]
    fn find_proxy() -> Result<(), Box<dyn std::error::Error>> {
        let mut eval = Evaluator::new(TEST_PAC_SCRIPT)?;
        assert_eq!(
            eval.find_proxy(&"http://localhost:3128/".parse::<Uri>().unwrap())?,
            Proxies::new(vec![ProxyDesc::Direct])
        );
        Ok(())
    }

    #[test]
    fn dns_resolve() -> Result<(), Box<dyn std::error::Error>> {
        let mut eval = Evaluator::new(TEST_PAC_SCRIPT)?;
        assert_ne!(eval.dns_resolve("localhost"), None);
        assert_eq!(eval.dns_resolve("thishostdoesnotexist"), None);
        Ok(())
    }
}
