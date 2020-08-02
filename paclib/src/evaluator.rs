use crate::Proxies;
use duktape::{Context, ContextRef, Stack};
use http::Uri;
use std::fmt::{Error, Formatter};
use std::result::Result;

const PAC_UTILS: &str = include_str!("pac_utils.js");

pub struct Evaluator {
    js: Context,
}

// To register:
// let idx = duk_push_c_function(ctx, /* func pointer */ dns_resolve, /* nargs: */ 1);
// duk_put_global_string(ctx, "dnsResolve");
unsafe extern "C" fn dns_resolve(ctx: *mut duktape_sys::duk_context) -> i32 {
    use std::net::ToSocketAddrs;
    let mut ctx = ContextRef::from(ctx);
    ctx.require_stack(1);
    eprintln!("dns_resolve");

    let value = if let Ok(host) = ctx.get_string(-1) {
        eprintln!("dnsResolve({})", host);
        let host_port = (&host[..], 0u16);
        if let Ok(mut addrs) = host_port.to_socket_addrs() {
            eprintln!("addres: {:?}", addrs);

            if let Some(addr) = addrs.next() {
                Some(addr.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    eprintln!("dnsResolve -> {:?}", value);

    if let Some(value) = value {
        if !ctx.push_string(&value).is_ok() {
            ctx.push_null();
        }
    } else {
        ctx.push_null();
    }
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
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
