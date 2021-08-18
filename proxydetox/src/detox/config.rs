#[derive(Debug, Clone)]
pub struct Config {
    /// sets the maximum idle connection per host allowed in the pool
    pub pool_max_idle_per_host: usize,

    /// set an optional timeout for idle sockets being kept-aliv.
    pub pool_idle_timeout: Option<std::time::Duration>,

    /// use the CONNECT method even for HTTP requests.
    pub always_use_connect: bool,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            pool_max_idle_per_host: usize::MAX,
            pool_idle_timeout: None,
            always_use_connect: false,
        }
    }
}
