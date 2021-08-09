pub mod bindings {
    ::windows::include_bindings!();
}

pub use bindings::Windows::Win32::Security::{
    Authentication::Identity::Core::{
        AcquireCredentialsHandleW, InitializeSecurityContextW, SecBuffer, SecBufferDesc,
        ISC_REQ_MUTUAL_AUTH, SECBUFFER_TOKEN, SECBUFFER_VERSION, SECPKG_CRED_OUTBOUND,
        SECURITY_NATIVE_DREP,
    },
    Credentials::SecHandle,
};
pub use windows::Error;
pub use windows::HRESULT;
