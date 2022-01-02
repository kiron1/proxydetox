use crate::Proxies;
use crate::{Uri, DNS_RESOLVE_NAME};
use parking_lot::{Condvar, Mutex};
use quick_js::{Context, JsValue};
use std::result::Result;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use tokio::sync::oneshot;

const PAC_UTILS: &str = include_str!("pac_utils.js");

#[derive(thiserror::Error, Debug)]
pub enum CreateEvaluatorError {
    #[error("failed to create JS context: {0}")]
    CreateContext(
        #[from]
        #[source]
        quick_js::ContextError,
    ),
    #[error("failed to create worker thread: {0}")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
    #[error("failed to evaluate PAC: {0}")]
    EvalPacFile(
        #[from]
        #[source]
        quick_js::ExecutionError,
    ),
    #[error(transparent)]
    ValueError(#[from] quick_js::ValueError),
}

#[derive(thiserror::Error, Debug)]
pub enum FindProxyError {
    #[error("no host in URL")]
    NoHost,
    #[error("invalid result from PAC script")]
    InvalidResult,
    #[error("internal error when calling FindProxyForURL function: {0}")]
    InternalError(
        #[from]
        #[source]
        quick_js::ExecutionError,
    ),
    #[error("shuting down")]
    Shutdown,
}

type FindProxyForURLRequest = (Uri, oneshot::Sender<FindProxyForURLResult>);
type FindProxyForURLResult = Result<Proxies, FindProxyError>;
type CreateEvaluatorResult = Option<Result<(), CreateEvaluatorError>>;

pub struct Evaluator {
    worker: Option<thread::JoinHandle<()>>,
    tx: Mutex<Option<std::sync::mpsc::Sender<FindProxyForURLRequest>>>,
}

impl Evaluator {
    pub fn new(pac_script: &str) -> Result<Self, CreateEvaluatorError> {
        // Simplify sending the pac_script to the worker thread.
        let pac_script = pac_script.to_owned();
        let create_js = Arc::new((Mutex::new(None as CreateEvaluatorResult), Condvar::new()));
        let create_js2 = create_js.clone();
        let (tx, rx) = mpsc::channel::<FindProxyForURLRequest>();
        let tx = Mutex::new(Some(tx));

        let worker = thread::Builder::new()
            .name("PAC Evaluator worker (quickjs)".into())
            .spawn(move || {
                let js = Context::new()
                    .map_err(CreateEvaluatorError::from)
                    .and_then(|js| {
                        js.add_callback(DNS_RESOLVE_NAME, crate::dns::Resolver)?;
                        js.eval(PAC_UTILS).expect("evaluation of PAC_UTILS");
                        js.eval(&pac_script)?;
                        Ok(js)
                    });

                {
                    // wait till the parent thread is ready and is waiting on the condition variable
                    thread::park();
                    // Notify the caller if we succedded in creating the JavaScript context
                    let (ref lock, ref cvar) = &*create_js2;
                    let mut result = lock.lock();

                    match js {
                        Ok(ref _cx) => *result = Some(Ok(())),
                        Err(cause) => {
                            *result = Some(Err(cause));
                            cvar.notify_one();
                            return;
                        }
                    }
                    let woke_up_one = cvar.notify_one();
                    assert!(woke_up_one);
                }

                let js = js.expect("js is Ok");

                while let Ok((uri, tx)) = rx.recv() {
                    let result = uri.host().ok_or(FindProxyError::NoHost).and_then(|host| {
                        let args = vec![JsValue::from(uri.to_string()), JsValue::from(host)];
                        let result = js.call_function("FindProxyForURL", args)?;
                        let result = result.as_str().ok_or(FindProxyError::InvalidResult)?;
                        Proxies::parse(result).map_err(|_| FindProxyError::InvalidResult)
                    });

                    tx.send(result).expect("send result");
                }
            })?;

        // Wait for the creation of the JavaScript context
        let result = {
            let (ref lock, ref cvar) = &*create_js;
            let mut result = lock.lock();
            worker.thread().unpark();

            if result.is_none() {
                cvar.wait(&mut result);
            }

            result.take()
        };

        result.expect("optinal has value").map(move |_| Evaluator {
            worker: Some(worker),
            tx,
        })
    }

    pub async fn find_proxy(&self, uri: Uri) -> Result<Proxies, FindProxyError> {
        let (tx, rx) = oneshot::channel::<FindProxyForURLResult>();
        {
            let call = self.tx.lock();
            if let Some(ref call) = *call {
                call.send((uri, tx)).expect("send");
            } else {
                return Err(FindProxyError::Shutdown);
            }
        }
        rx.await.expect("receive")
    }
}

impl Drop for Evaluator {
    fn drop(&mut self) {
        {
            *self.tx.lock() = None;
        }
        let _join = self.worker.take().expect("worker").join();
    }
}

#[cfg(test)]
mod tests {
    use super::Evaluator;
    use super::Uri;
    use crate::Proxies;
    use crate::ProxyDesc;

    const TEST_PAC_SCRIPT: &str = "function FindProxyForURL(url, host) { return \"DIRECT\"; }";

    #[tokio::test]
    async fn find_proxy() -> Result<(), Box<dyn std::error::Error>> {
        let eval = Evaluator::new(TEST_PAC_SCRIPT)?;
        assert_eq!(
            eval.find_proxy("http://localhost:3128/".parse::<Uri>().unwrap())
                .await?,
            Proxies::new(vec![ProxyDesc::Direct])
        );
        Ok(())
    }
}
