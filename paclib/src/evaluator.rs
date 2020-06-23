use crate::Proxies;
use duktape::Context;
use duktape_sys::duk_context;
use std::fmt::{Error, Formatter};
use std::result::Result;
use url::Url;

const PAC_UTILS: &str = include_str!("pac_utils.js");

pub struct Evaluator {
    js: Context,
}

// To register:
// let idx = duk_push_c_function(ctx, /* func pointer */ dns_resolve, /* nargs: */ 1);
// duk_put_global_string(ctx, "dnsResolve");
//unsafe extern "C" fn dns_resolve(ctx: *mut duk_context) -> () {
//    use std::net::ToSocketAddrs;
//    let host = "asdf";
//    let host_port = (host, 0);
//    if let Ok(ref mut ip_iter) = host_port.to_socket_addrs() {
//        if let Some(ref mut addr) = ip_iter.next() {
//            // TODO push string on context
//            let s = appr.to_string();
//        }
//    }
//}

impl Evaluator {
    pub fn new(pac_script: &str) -> Result<Self, CreateEvaluatorError> {
        let mut ctx = Context::new().map_err(|_| CreateEvaluatorError::CreateContext)?;
        ctx.eval(PAC_UTILS).expect("eval pac_utils.js");
        ctx.eval(pac_script)
            .map_err(|_| CreateEvaluatorError::EvalPacFile)?;
        Ok(Evaluator { js: ctx })
    }

    pub fn find_proxy(&mut self, url: &Url) -> Result<Proxies, FindProxyError> {
        let host = url.host_str().ok_or(FindProxyError::NoHost)?;
        //let result = self.js.eval(&format!(
        //    "FindProxyForURL(\"{}\", \"{}\");",
        //    url.as_str(),
        //    host
        //));
        // FIXME: when something goes wrong here we need to clean up the stack!
        let result = {
            self.js
                .get_global_string("FindProxyForURL")
                .map_err(|_| FindProxyError::InternalError)?;
            self.js
                .push_string(url.as_str())
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
    use crate::Proxies;
    use crate::ProxyDesc;
    use url::Url;

    const TEST_PAC_SCRIPT: &str = "function FindProxyForURL(url, host) { return \"DIRECT\"; }";
    #[test]
    fn find_proxy() -> Result<(), Box<dyn std::error::Error>> {
        let mut eval = Evaluator::new(TEST_PAC_SCRIPT)?;
        assert_eq!(
            eval.find_proxy(&Url::parse("http://localhost:3128/").unwrap())?,
            Proxies::new(vec![ProxyDesc::Direct])
        );
        Ok(())
    }
}
