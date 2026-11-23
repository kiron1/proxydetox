use http::Uri;
use std::convert::Infallible;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use tokio::sync::oneshot;

use crate::Proxies;
use crate::engine::Engine;
use crate::{FindProxyError, PacScriptError};

pub struct Evaluator {
    _worker: Arc<thread::JoinHandle<()>>,
    sender: Mutex<Option<mpsc::Sender<Action>>>,
}

type FindProxyResult = Result<Proxies, FindProxyError>;
type SetPacScriptResult = Result<(), PacScriptError>;
type SetMyIpAddressResult = Result<(), Infallible>;

enum Action {
    FindProxy(Uri, oneshot::Sender<FindProxyResult>),
    SetPacScript(Option<String>, oneshot::Sender<SetPacScriptResult>),
    SetMyIpAddress(IpAddr, oneshot::Sender<SetMyIpAddressResult>),
}

impl Evaluator {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<Action>();

        let worker = thread::Builder::new()
            .name("pac-eval-worker".into())
            .spawn(move || Self::run(receiver, None, None))
            .expect("create thread");

        Self {
            _worker: Arc::new(worker),
            sender: Mutex::new(Some(sender)),
        }
    }

    pub fn with_pac_script(pac_script: &str) -> Result<Self, PacScriptError> {
        let pac_script = pac_script.to_owned();
        let (sender, receiver) = mpsc::channel::<Action>();
        let (initialized_sender, initialized_receiver) = mpsc::sync_channel(1);

        let worker = thread::Builder::new()
            .name("pac-eval-worker".into())
            .spawn(move || Self::run(receiver, Some(pac_script), Some(initialized_sender)))
            .expect("create thread");

        initialized_receiver.recv().map_err(|_| {
            PacScriptError::InternalError(
                "PAC evaluator worker stopped during initialization".into(),
            )
        })??;

        Ok(Self {
            _worker: Arc::new(worker),
            sender: Mutex::new(Some(sender)),
        })
    }

    fn run(
        receiver: mpsc::Receiver<Action>,
        pac_script: Option<String>,
        initialized_sender: Option<mpsc::SyncSender<SetPacScriptResult>>,
    ) {
        let mut engine = Engine::new();
        if let Some(initialized_sender) = initialized_sender {
            let result = engine.set_pac_script(pac_script.as_deref());
            let failed = result.is_err();
            if initialized_sender.send(result).is_err() || failed {
                return;
            }
        }

        while let Ok(action) = receiver.recv() {
            match action {
                Action::FindProxy(ref uri, result) => {
                    let r = engine.find_proxy(uri);
                    result.send(r).ok();
                }
                Action::SetPacScript(ref script, result) => {
                    let r = engine.set_pac_script(script.as_deref());
                    result.send(r).ok();
                }
                Action::SetMyIpAddress(addr, result) => {
                    let r = engine.set_my_ip_address(addr);
                    result.send(r).ok();
                }
            }
        }
    }

    pub async fn find_proxy(&self, uri: Uri) -> FindProxyResult {
        let (tx, rx) = oneshot::channel::<FindProxyResult>();
        {
            let sender = self.sender.lock().unwrap();
            if let Some(ref sender) = *sender {
                sender.send(Action::FindProxy(uri, tx)).expect("send");
            }
        }
        rx.await.expect("receive")
    }

    pub async fn set_pac_script(&self, pac_script: Option<String>) -> SetPacScriptResult {
        let (tx, rx) = oneshot::channel::<SetPacScriptResult>();
        {
            let sender = self.sender.lock().unwrap();
            if let Some(ref sender) = *sender {
                sender
                    .send(Action::SetPacScript(pac_script, tx))
                    .expect("send");
            }
        }
        rx.await.expect("receive")
    }

    pub async fn set_my_ip_address(&self, addr: IpAddr) -> SetMyIpAddressResult {
        let (tx, rx) = oneshot::channel::<SetMyIpAddressResult>();
        {
            let sender = self.sender.lock().unwrap();
            if let Some(ref sender) = *sender {
                sender.send(Action::SetMyIpAddress(addr, tx)).expect("send");
            }
        }
        rx.await.expect("receive")
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Evaluator {
    fn drop(&mut self) {
        let mut sender = self.sender.lock().unwrap();
        let _ = sender.take();
        // self.worker.join();
    }
}

#[cfg(test)]
mod tests {
    use super::Evaluator;
    use crate::{PacScriptError, Proxies, ProxyOrDirect};

    #[test]
    fn with_pac_script_reports_syntax_errors() {
        let result = Evaluator::with_pac_script("function FindProxyForURL(url, host) {");

        assert!(matches!(result, Err(PacScriptError::InternalError(_))));
    }

    #[test]
    fn with_pac_script_reports_missing_entry_function() {
        let result = Evaluator::with_pac_script(
            "function FindProxyForUrl(url, host) { return \"DIRECT\"; }",
        );

        assert!(matches!(
            result,
            Err(PacScriptError::FindProxyForURLMissing)
        ));
    }

    #[tokio::test]
    async fn new_keeps_the_default_pac_function() {
        let evaluator = Evaluator::new();

        assert_eq!(
            evaluator
                .find_proxy("http://localhost/".parse().unwrap())
                .await
                .unwrap(),
            Proxies::new(vec![ProxyOrDirect::Direct])
        );
    }
}
