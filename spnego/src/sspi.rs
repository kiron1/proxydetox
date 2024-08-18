use std::ffi::c_void;

use windows::core::PCWSTR;
use windows::Win32::Security::{
    Authentication::Identity::{
        AcquireCredentialsHandleW, InitializeSecurityContextW, SecBuffer, SecBufferDesc,
        ISC_REQ_MUTUAL_AUTH, SECBUFFER_TOKEN, SECBUFFER_VERSION, SECPKG_CRED_OUTBOUND,
        SECURITY_NATIVE_DREP,
    },
    Credentials::SecHandle,
};

type TimeStamp = i64;

// https://github.com/java-native-access/jna/issues/261
const MAX_TOKEN_SIZE: usize = 48 * 1024;

pub(super) struct Context {
    cx: SecHandle,
    cred: SecHandle,
    target: String,
    spn: Vec<u16>,
    expiry: TimeStamp,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("target", &self.target)
            .field("spn", &self.spn)
            .field("expiry", &self.expiry)
            .finish()
    }
}

fn to_utf16(value: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(value).encode_wide().chain(once(0u16)).collect()
}

impl Context {
    pub(super) fn new(
        service: &str,
        proxy_fqdn: &str,
    ) -> std::result::Result<Self, windows::core::Error> {
        let target = format!("{service}/{proxy_fqdn}");
        let spn = to_utf16(&target);

        let package = to_utf16("Negotiate");
        let mut cred = SecHandle::default();
        let mut expiry = TimeStamp::default();

        // https://docs.microsoft.com/en-us/windows/win32/secauthn/acquirecredentialshandle--kerberos
        unsafe {
            AcquireCredentialsHandleW(
                None,
                PCWSTR(package.as_ptr()),
                SECPKG_CRED_OUTBOUND,
                None,
                None,
                None,
                None,
                &mut cred,
                Some(&mut expiry),
            )?
        };

        let cx = SecHandle::default();

        Ok(Self {
            cx,
            target,
            spn,
            cred,
            expiry,
        })
    }

    // TODO: server token is not used right now.
    // https://docs.microsoft.com/en-us/openspecs/office_protocols/ms-grvhenc/b9e676e7-e787-4020-9840-7cfe7c76044a?redirectedfrom=MSDN
    // https://docs.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc772815(v=ws.10)
    pub(super) fn step(
        &mut self,
        _server_token: Option<&[u8]>,
    ) -> std::result::Result<Option<Vec<u8>>, windows::core::Error> {
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
                Some(&mut self.cred),
                None,
                Some(self.spn.as_ptr()),
                ISC_REQ_MUTUAL_AUTH,
                0,
                SECURITY_NATIVE_DREP,
                None,
                0,
                Some(&mut self.cx),
                Some(&mut buffer_desc),
                &mut cx_attrs,
                Some(&mut self.expiry),
            )
        };
        status.ok()?;

        // Shrink buffer to acutall token size
        buf.resize(sec_buffer[0].cbBuffer as usize, 0);

        if buf.len() > 0 {
            Ok(Some(buf))
        } else {
            Ok(None)
        }
    }
}
