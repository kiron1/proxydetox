pub use windows::core::{Error, HRESULT};
pub use windows::Win32::Foundation::PWSTR;
pub use windows::Win32::Security::{
    Authentication::Identity::{
        AcquireCredentialsHandleW, InitializeSecurityContextW, SecBuffer, SecBufferDesc,
        ISC_REQ_MUTUAL_AUTH, SECBUFFER_TOKEN, SECBUFFER_VERSION, SECPKG_CRED_OUTBOUND,
        SECURITY_NATIVE_DREP,
    },
    Credentials::SecHandle,
};
