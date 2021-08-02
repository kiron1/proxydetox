mod bindings {
    ::windows::include_bindings!();
}

use std::{ffi::c_void, ptr::null_mut};

use bindings::Windows::Win32::Security::{
    Authentication::Identity::Core::{
        AcquireCredentialsHandleW, InitializeSecurityContextW, SecBuffer, SecBufferDesc,
        ISC_REQ_MUTUAL_AUTH, SECBUFFER_TOKEN, SECBUFFER_VERSION, SECPKG_CRED_OUTBOUND,
        SECURITY_NATIVE_DREP,
    },
    Credentials::SecHandle,
};
use windows::HRESULT;

type TimeStamp = i64;

// https://github.com/java-native-access/jna/issues/261
const MAX_TOKEN_SIZE: usize = 48 * 1024;

#[derive(Debug)]
pub struct Context {
    spn: Vec<u16>,
    cred: SecHandle,
    cx: SecHandle,
    expiry: i64,
}

fn to_utf16(value: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(value).encode_wide().chain(once(0u16)).collect()
}

impl Context {
    pub fn new(proxy_fqdn: &str) -> windows::Result<Self> {
        let spn = format!("HTTP/{}", proxy_fqdn);
        dbg!(&spn);
        let spn = to_utf16(&spn);

        let package = "Negotiate";
        let mut cred = SecHandle::default();
        let mut expiry = TimeStamp::default();
        // https://docs.microsoft.com/en-us/windows/win32/secauthn/acquirecredentialshandle--kerberos
        let status = unsafe {
            AcquireCredentialsHandleW(
                None,
                package,
                SECPKG_CRED_OUTBOUND,
                null_mut(),
                null_mut(),
                None,
                null_mut(),
                &mut cred,
                &mut expiry,
            )
        };
        let status = HRESULT(status as _);
        status.ok()?;

        let cx = SecHandle::default();

        Ok(Self {
            cx,
            spn,
            cred,
            expiry,
        })
    }

    pub fn step(&mut self) -> windows::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(MAX_TOKEN_SIZE);
        buf.resize(MAX_TOKEN_SIZE, 0);
        let mut sec_buffer = [SecBuffer {
            BufferType: SECBUFFER_TOKEN,
            cbBuffer: MAX_TOKEN_SIZE as u32,
            pvBuffer: buf.as_mut_ptr() as *mut c_void,
        }];
        let mut buffer_desc = SecBufferDesc {
            ulVersion: SECBUFFER_VERSION,
            cBuffers: sec_buffer.len() as u32,
            pBuffers: sec_buffer.as_mut_ptr(),
        };
        let mut cx_attrs = 0u32;
        // https://docs.microsoft.com/en-us/windows/win32/api/sspi/nf-sspi-initializesecuritycontexta
        let status = unsafe {
            InitializeSecurityContextW(
                &mut self.cred,
                null_mut(),
                self.spn.as_mut_ptr(),
                ISC_REQ_MUTUAL_AUTH,
                0,
                SECURITY_NATIVE_DREP,
                null_mut(),
                0,
                &mut self.cx,
                &mut buffer_desc,
                &mut cx_attrs,
                &mut self.expiry,
            )
        };
        let status = HRESULT(status as _);
        status.ok()?;

        // Shrink buffer to acutall token size
        buf.resize(sec_buffer[0].cbBuffer as usize, 0);

        Ok(buf)
    }
}
