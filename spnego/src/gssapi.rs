use libgssapi::{
    context::{ClientCtx, CtxFlags},
    name::Name,
    oid::{Oid, GSS_NT_HOSTBASED_SERVICE},
};
use std::result::Result;

// GSS_SPNEGO_MECHANISM - 1.3.6.1.5.5.2
static GSS_MECH_SPNEGO: Oid = Oid::from_slice(b"\x2b\x06\x01\x05\x05\x02");

#[derive(Debug)]
pub struct Context {
    cx: ClientCtx,
}

impl Context {
    pub fn new(service: &str, proxy_fqdn: &str) -> Result<Self, libgssapi::error::Error> {
        // GSS-API uses service@host format, while
        // Kerberos SPN uses service/host@REALM (@REALM is optional).
        let target = format!("{service}@{proxy_fqdn}");
        let service_name = target.as_bytes();

        let name = Name::new(service_name, Some(&GSS_NT_HOSTBASED_SERVICE))?;
        // let name = name.canonicalize(Some(&GSS_MECH_SPNEGO))?;

        let cx = ClientCtx::new(
            None,
            name,
            CtxFlags::GSS_C_MUTUAL_FLAG,
            Some(&GSS_MECH_SPNEGO),
        );

        Ok(Self { cx })
    }

    // Call `step` `while request.status() == http::StatusCode::PROXY_AUTHENTICATION_REQUIRED {}`.
    pub fn step(
        &mut self,
        server_token: Option<&[u8]>,
    ) -> Result<Option<Vec<u8>>, libgssapi::error::Error> {
        let token = self.cx.step(server_token, None);
        match token {
            Ok(Some(token)) => Ok(Some(Vec::from(&*token))),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
