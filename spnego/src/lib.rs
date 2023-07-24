#[cfg(unix)]
pub mod gssapi;

#[cfg(windows)]
pub mod sspi;

#[cfg(unix)]
pub use gssapi::Context;

#[cfg(windows)]
pub use sspi::Context;
