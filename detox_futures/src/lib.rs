pub trait FutureExt: std::future::Future {
    fn timeout(self, dt: std::time::Duration) -> tokio::time::Timeout<Self>
    where
        Self: Sized,
    {
        tokio::time::timeout(dt, self)
    }
}

impl<T: ?Sized> FutureExt for T where T: std::future::Future {}
