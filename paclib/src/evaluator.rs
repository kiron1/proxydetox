use crate::DnsCache;
use crate::Proxies;
use crate::{DNS_CACHE_NAME, DNS_RESOLVE_NAME};
use duktape::{Context, Stack};
use http::Uri;
use std::result::Result;

const PAC_UTILS: &str = include_str!("pac_utils.js");

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum CreateEvaluatorError {
    #[error("failed to create JS context")]
    CreateContext,
    #[error("failed to evaluate PAC: {0}")]
    EvalPacFile(
        #[from]
        #[source]
        PacScriptError,
    ),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
#[error("Invalid PAC script")]
pub struct PacScriptError;

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum FindProxyError {
    #[error("no host in URL")]
    NoHost,
    #[error("invalid result from PAC script")]
    InvalidResult,
    #[error("internal error when processing PAC script")]
    InternalError,
}
pub struct Evaluator {
    js: Context,
    dns_cache: Box<DnsCache>,
}

impl Evaluator {
    pub fn new() -> Result<Self, CreateEvaluatorError> {
        let mut ctx = Context::new().map_err(|_| CreateEvaluatorError::CreateContext)?;
        let mut dns_cache: Box<DnsCache> = Default::default();

        ctx.put_global_pointer(DNS_CACHE_NAME, dns_cache.as_mut() as *mut _)
            .map_err(|_| CreateEvaluatorError::CreateContext)?;

        ctx.push_c_function(DNS_RESOLVE_NAME, crate::dns::dns_resolve, 1)
            .map_err(|_| CreateEvaluatorError::CreateContext)?;

        ctx.eval(PAC_UTILS).expect("eval pac_utils.js");
        ctx.eval(crate::DEFAULT_PAC_SCRIPT)
            .expect("eval default PAC script");

        Ok(Evaluator { js: ctx, dns_cache })
    }

    pub fn with_pac_script(pac_script: &str) -> Result<Self, CreateEvaluatorError> {
        let mut new = Self::new()?;
        new.set_pac_script(Some(pac_script))?;
        Ok(new)
    }

    pub fn set_pac_script(&mut self, pac_script: Option<&str>) -> Result<(), PacScriptError> {
        let pac_script = pac_script.unwrap_or(crate::DEFAULT_PAC_SCRIPT);
        self.js.eval(pac_script).map_err(|_| PacScriptError)?;
        Ok(())
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
            Ok(duktape::Value::String(ref result)) => Ok(result
                .parse::<Proxies>()
                .map_err(|_| FindProxyError::InvalidResult)?),
            _ => Err(FindProxyError::InvalidResult),
        }
    }

    pub fn cache(&mut self) -> crate::dns::DnsMap {
        self.dns_cache.map()
    }

    #[cfg(test)]
    fn dns_resolve(&mut self, host: &str) -> Option<String> {
        // FIXME: when something goes wrong here we need to clean up the stack!
        let result = {
            self.js.get_global_string(DNS_RESOLVE_NAME).unwrap();
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

#[cfg(test)]
mod tests {
    use super::Evaluator;
    use super::Uri;
    use crate::Proxies;
    use crate::ProxyDesc;

    #[test]
    fn find_proxy() -> Result<(), Box<dyn std::error::Error>> {
        let mut eval = Evaluator::new()?;
        assert_eq!(
            eval.find_proxy(&"http://localhost:3128/".parse::<Uri>().unwrap())?,
            Proxies::new(vec![ProxyDesc::Direct])
        );
        Ok(())
    }

    #[test]
    fn dns_resolve() -> Result<(), Box<dyn std::error::Error>> {
        let mut eval = Evaluator::new()?;
        assert_ne!(eval.dns_resolve("localhost"), None);
        assert_eq!(eval.dns_resolve("thishostdoesnotexist."), None);
        Ok(())
    }
}
