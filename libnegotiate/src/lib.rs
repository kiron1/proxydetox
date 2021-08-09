#[cfg(all(feature = "gssapi", target_family = "unix"))]
pub mod gssapi;
#[cfg(target_family = "windows")]
pub mod sspi;

#[cfg(all(feature = "gssapi", target_family = "unix"))]
pub use gssapi::Context;
#[cfg(target_family = "windows")]
pub use sspi::Context;
