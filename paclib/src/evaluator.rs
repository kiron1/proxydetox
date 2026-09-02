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
    SetPacScripts(Vec<String>, oneshot::Sender<SetPacScriptResult>),
    SetMyIpAddress(IpAddr, oneshot::Sender<SetMyIpAddressResult>),
}

impl Evaluator {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<Action>();

        let worker = thread::Builder::new()
            .name("pac-eval-worker".into())
            .spawn(move || Self::run(receiver, Vec::new()))
            .expect("create thread");

        Self {
            _worker: Arc::new(worker),
            sender: Mutex::new(Some(sender)),
        }
    }

    pub fn with_pac_script(pac_script: &str) -> Result<Self, PacScriptError> {
        Self::with_pac_scripts(vec![pac_script.to_owned()])
    }

    pub fn with_pac_scripts(pac_scripts: Vec<String>) -> Result<Self, PacScriptError> {
        let (sender, receiver) = mpsc::channel::<Action>();

        let initial_scripts = pac_scripts.clone();
        let worker = thread::Builder::new()
            .name("pac-eval-worker".into())
            .spawn(move || Self::run(receiver, initial_scripts))
            .expect("create thread");

        let new = Self {
            _worker: Arc::new(worker),
            sender: Mutex::new(Some(sender)),
        };
        Ok(new)
    }

    fn run(receiver: mpsc::Receiver<Action>, pac_scripts: Vec<String>) {
        let mut engines = Self::engines(pac_scripts).unwrap_or_default();

        while let Ok(action) = receiver.recv() {
            match action {
                Action::FindProxy(ref uri, result) => {
                    let r = engines
                        .iter_mut()
                        .map(|engine| engine.find_proxy(uri))
                        .find(|result| {
                            result
                                .as_ref()
                                .map(|proxies| {
                                    proxies
                                        .iter()
                                        .any(|proxy| !matches!(proxy, crate::ProxyOrDirect::Direct))
                                })
                                .unwrap_or(true)
                        })
                        .unwrap_or_else(|| Ok(crate::Proxies::direct()));
                    result.send(r).ok();
                }
                Action::SetPacScripts(scripts, result) => {
                    let r = Self::engines(scripts).map(|new| {
                        engines = new;
                    });
                    result.send(r).ok();
                }
                Action::SetMyIpAddress(addr, result) => {
                    let r = engines
                        .iter_mut()
                        .try_for_each(|engine| engine.set_my_ip_address(addr));
                    result.send(r).ok();
                }
            }
        }
    }

    fn engines(scripts: Vec<String>) -> Result<Vec<Engine>, PacScriptError> {
        scripts
            .into_iter()
            .map(|script| Engine::with_pac_script(&script))
            .collect()
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
        self.set_pac_scripts(pac_script.into_iter().collect()).await
    }

    pub async fn set_pac_scripts(&self, pac_scripts: Vec<String>) -> SetPacScriptResult {
        let (tx, rx) = oneshot::channel::<SetPacScriptResult>();
        {
            let sender = self.sender.lock().unwrap();
            if let Some(ref sender) = *sender {
                sender
                    .send(Action::SetPacScripts(pac_scripts, tx))
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
    use crate::{Proxy, ProxyOrDirect};

    #[tokio::test]
    async fn uses_first_pac_file_returning_a_proxy() {
        let evaluator = Evaluator::with_pac_scripts(vec![
            "function FindProxyForURL(url, host) { return \"DIRECT\"; }".into(),
            "function FindProxyForURL(url, host) { return \"PROXY proxy.example:8080\"; }".into(),
        ])
        .unwrap();

        assert_eq!(
            evaluator
                .find_proxy("http://example.org/".parse().unwrap())
                .await
                .unwrap(),
            crate::Proxies::new(vec![ProxyOrDirect::Proxy(Proxy::Http(
                "proxy.example:8080".parse().unwrap()
            ))])
        );
    }
}
