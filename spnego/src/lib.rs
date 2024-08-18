#[cfg(unix)]
mod gssapi;

#[cfg(windows)]
mod sspi;

#[derive(Debug)]
pub struct Context {
    #[cfg(unix)]
    inner: gssapi::Context,

    #[cfg(windows)]
    inner: sspi::Context,
}

impl Context {
    pub fn new(service: &str, proxy_fqdn: &str) -> crate::Result<Self> {
        #[cfg(unix)]
        {
            Ok(Self {
                inner: gssapi::Context::new(service, proxy_fqdn)
                    .map_err(|inner| Error { inner })?,
            })
        }

        #[cfg(windows)]
        {
            Ok(Self {
                inner: sspi::Context::new(service, proxy_fqdn).map_err(|inner| Error { inner })?,
            })
        }
    }

    pub fn step(&mut self, server_token: Option<&[u8]>) -> crate::Result<Option<Vec<u8>>> {
        self.inner
            .step(server_token)
            .map_err(|inner| Error { inner })
    }
}

#[derive(Debug)]
pub struct Error {
    #[cfg(unix)]
    inner: libgssapi::error::Error,

    #[cfg(windows)]
    inner: windows::core::Error,
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SPNEGO error: ")?;
        self.inner.fmt(f)
    }
}
