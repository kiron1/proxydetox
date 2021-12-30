#[cfg(target_family = "windows")]
mod win;

#[cfg(target_family = "windows")]
pub use win::{
    AcquireCredentialsHandleW, Error, InitializeSecurityContextW, SecBuffer, SecBufferDesc,
    SecHandle, HRESULT, ISC_REQ_MUTUAL_AUTH, PWSTR, SECBUFFER_TOKEN, SECBUFFER_VERSION,
    SECPKG_CRED_OUTBOUND, SECURITY_NATIVE_DREP,
};
