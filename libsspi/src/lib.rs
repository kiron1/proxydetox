mod bindings {
    ::windows::include_bindings!();
}

use std::ptr::null_mut;

use bindings::bindings::windows::win32::{
    security::{
        AcquireCredentialsHandleW, InitializeSecurityContextW, SecHandle, SECPKG_CRED_OUTBOUND,
    },
    system_services::LARGE_INTEGER,
};

type TimeStamp = LARGE_INTEGER;

pub struct Context {
    spn: String,
    cred: SecHandle,
}

impl Context {
    pub fn new(proxy_url: &http::Uri) -> Self {
        let spn = format!("HTTP/{}", proxy_url.host().expect("URL with host"));
        let kerberos = std::ffi::OsString::from("Kerberos");
        let mut cred: SecHandle;
        let mut expiry: TimeStamp;
        // https://docs.microsoft.com/en-us/windows/win32/secauthn/acquirecredentialshandle--kerberos
        let status = AcquireCredentialsHandle(
            null_mut(),
            &kerberos,
            SECPKG_CRED_OUTBOUND,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            &mut cred,
            &mut expiry,
        );

        Self { spn, cred }
    }

    pub fn step(&mut self) {
        let mut cx: SecHandle;

        // https://docs.microsoft.com/en-us/windows/win32/api/sspi/nf-sspi-initializesecuritycontexta
        let status = InitializeSecurityContextW(
            &self.cred,
            null_mut(),
            ISC_REQ_MUTUAL_AUTH,
            0,
            SECURITY_NATIVE_DREP,
            bufdec,
            0,
            cx,
            null_mut(),
            null_mut(),
            null_mut(),
        );
    }
}
