use std::{pin::Pin, sync::Arc};

use parking_lot::Mutex;
use tokio::sync::oneshot;

pub fn idle_timeout(idle_timeout: std::time::Duration) -> (KeepAliveToken, Signal) {
    let (tx, rx) = oneshot::channel::<()>();
    (KeepAliveToken(Inner::new(idle_timeout, tx)), Signal(rx))
}

#[derive(Debug)]
pub struct KeepAliveToken(Arc<Inner>);

impl Clone for KeepAliveToken {
    fn clone(&self) -> Self {
        let mut inner = self.0.signal.lock();
        // abort any running timeout
        if let Some(h) = inner.hande.take() {
            h.abort()
        }
        inner.use_count += 1;
        KeepAliveToken(self.0.clone())
    }
}

impl Drop for KeepAliveToken {
    fn drop(&mut self) {
        let mut signal = self.0.signal.lock();
        signal.use_count -= 1;
        if signal.use_count == 0 {
            let handle = tokio::spawn(timed_trigger(self.0.clone()));
            signal.hande = Some(handle);
        }
    }
}

pub struct Signal(oneshot::Receiver<()>);

#[derive(Debug)]
struct Inner {
    timeout: std::time::Duration,
    signal: Mutex<IdleSignal>,
}

impl Inner {
    pub fn new(timeout: std::time::Duration, tx: oneshot::Sender<()>) -> Arc<Self> {
        let signal = Mutex::new(IdleSignal {
            use_count: 0,
            tx: Some(tx),
            hande: None,
        });
        let this = Arc::new(Self { signal, timeout });
        let handle = tokio::spawn(timed_trigger(this.clone()));
        this.signal.lock().hande = Some(handle);
        this
    }
}

#[derive(Debug)]
struct IdleSignal {
    use_count: i32, // number of active session
    tx: Option<oneshot::Sender<()>>,
    hande: Option<tokio::task::JoinHandle<()>>,
}

async fn timed_trigger(inner: Arc<Inner>) {
    tokio::time::sleep(inner.timeout).await;
    inner.signal.lock().tx.take().map(|tx| tx.send(()).ok());
}

impl std::future::Future for Signal {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(&mut self.get_mut().0).poll(cx).map(|_| ())
    }
}
