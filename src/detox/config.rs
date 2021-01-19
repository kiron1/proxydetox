#[derive(Debug, Clone)]
pub struct Config {
    /// sets the maximum idle connection per host allowed in the pool
    pub pool_max_idle_per_host: usize,

    /// set an optional timeout for idle sockets being kept-aliv.
    pub pool_idle_timeout: Option<std::time::Duration>,
}
