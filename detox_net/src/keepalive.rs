use std::time::Duration;

#[cfg(unix)]
use std::os::unix::io::AsFd;
#[cfg(windows)]
use std::os::windows::io::AsSocket as AsFd;

#[derive(Clone, Debug, Default)]
pub struct TcpKeepAlive {
    time: Option<Duration>,
    interval: Option<Duration>,
    retries: Option<u32>,
}

impl TcpKeepAlive {
    pub fn new() -> Self {
        Default::default()
    }

    pub const fn with_time(mut self, time: Option<Duration>) -> Self {
        self.time = time;
        self
    }

    pub const fn with_interval(mut self, interval: Option<Duration>) -> Self {
        self.interval = interval;
        self
    }

    pub const fn with_retries(mut self, retries: Option<u32>) -> Self {
        self.retries = retries;
        self
    }

    pub const fn time(&self) -> Option<Duration> {
        self.time
    }

    pub const fn interval(&self) -> Option<Duration> {
        self.interval
    }

    pub const fn retries(&self) -> Option<u32> {
        self.retries
    }

    pub fn apply<T: AsFd>(&self, socket: &T) -> std::io::Result<()> {
        let mut ka = socket2::TcpKeepalive::new();
        if let Some(t) = self.time {
            ka = ka.with_time(t);
        }
        if let Some(i) = self.interval {
            ka = ka.with_interval(i);
        }
        #[cfg(unix)]
        if let Some(r) = self.retries {
            ka = ka.with_retries(r);
        }
        let sock_ref = socket2::SockRef::from(socket);
        sock_ref.set_tcp_keepalive(&ka)
    }
}
