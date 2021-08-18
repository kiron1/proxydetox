use proxydetox::auth::AuthenticatorFactory;
use proxydetox::detox::Config;
use proxydetox::Server;
use std::ffi::CStr;
use std::path::PathBuf;

fn netrc_path() -> PathBuf {
    let mut netrc_path = dirs::home_dir().unwrap_or_default();
    netrc_path.push(".netrc");
    netrc_path.to_owned()
}

#[no_mangle]
pub extern "C" fn proxydetox_new(
    pac_script: *const libc::c_char,
    #[allow(unused_variables)] negotiate: bool,
    port: u16,
) -> *mut Server {
    let pac_script = unsafe { CStr::from_ptr(pac_script) };
    let pac_script = pac_script.to_str().unwrap_or_default().to_owned();

    #[cfg(feature = "negotiate")]
    let auth = if negotiate {
        AuthenticatorFactory::negotiate()
    } else {
        AuthenticatorFactory::basic(netrc_path())
    };

    #[cfg(not(feature = "negotiate"))]
    let auth = AuthenticatorFactory::basic(netrc_path());

    let config = Config::default();

    let server = Box::new(Server::new(pac_script, auth, port, config));

    Box::<Server>::into_raw(server)
}

#[no_mangle]
pub extern "C" fn proxydetox_run(server: *mut Server) {
    use tokio::runtime::Builder;

    let mut server = unsafe { Box::from_raw(server) };

    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("proxydetox-tokio-rt")
        .build()
        .unwrap();

    let _ = runtime.block_on(async { server.run().await });

    Box::leak(server);
}

#[no_mangle]
pub extern "C" fn proxydetox_drop(server: *mut Server) {
    unsafe { Box::from_raw(server) };
}
