use libgssapi::{
    context::{ClientCtx, CtxFlags},
    credential::{Cred, CredUsage},
    name::Name,
    oid::{OidSet, GSS_MECH_KRB5, GSS_NT_HOSTBASED_SERVICE},
};
use std::result::Result;

#[derive(Debug)]
pub struct Context {
    target: String,
}

impl Context {
    pub fn new(proxy_fqdn: &str) -> Result<Self, libgssapi::error::Error> {
        let target = format!("http@{}", proxy_fqdn);

        Ok(Self { target })
    }

    pub fn target_name(&self) -> &str {
        &self.target
    }

    fn make_client(target: &str) -> Result<ClientCtx, libgssapi::error::Error> {
        let desired_mechs = {
            let mut s = OidSet::new().expect("OidSet::new");
            s.add(&GSS_MECH_KRB5).expect("GSS_MECH_KRB5");
            s
        };

        let service_name = target.as_bytes();

        let name = Name::new(service_name, Some(&GSS_NT_HOSTBASED_SERVICE))?;
        let name = name.canonicalize(Some(&GSS_MECH_KRB5))?;

        let client_cred = Cred::acquire(None, None, CredUsage::Initiate, Some(&desired_mechs))?;

        Ok(ClientCtx::new(
            client_cred,
            name,
            CtxFlags::GSS_C_MUTUAL_FLAG,
            Some(&GSS_MECH_KRB5),
        ))
    }

    // Call `step` `while request.status() == http::StatusCode::PROXY_AUTHENTICATION_REQUIRED {}`.
    pub fn step(
        &self,
        server_token: Option<&[u8]>,
    ) -> Result<Option<Vec<u8>>, libgssapi::error::Error> {
        // todo: actually the client context should be persistent across calls to step.
        // but currently there is no way to know when the context is completed and a new one needs
        // to be created. therefor we always create a fresh one (which seems to work).

        // Get client token, and create new gss client context.
        let stepper = Self::make_client(self.target_name())?;
        let token = server_token.as_deref();
        let token = stepper.step(token);

        match token {
            Ok(Some(token)) => Ok(Some(Vec::from(&*token))),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
