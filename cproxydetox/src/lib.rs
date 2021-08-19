use proxydetox::auth::AuthenticatorFactory;
use proxydetox::detox::Config;
use proxydetox::http_file;
use proxydetox::Server;
use std::ffi::CStr;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::ptr::null_mut;

fn netrc_path() -> PathBuf {
    let mut netrc_path = dirs::home_dir().unwrap_or_default();
    netrc_path.push(".netrc");
    netrc_path.to_owned()
}

fn load_pac_file(path: &str) -> String {
    use tokio::runtime::Runtime;

    let content = if path.starts_with("http://") {
        if let Ok(url) = path.parse() {
            let rt = Runtime::new().unwrap();
            let content = rt.block_on(async { http_file(url).await.unwrap_or_default() });
            content
        } else {
            "".into()
        }
    } else {
        read_to_string(&path).unwrap_or_default()
    };

    if content.trim().is_empty() {
        "function FindProxyForURL(url, host) { return \"DIRECT\"; }".into()
    } else {
        content
    }
}

#[no_mangle]
pub extern "C" fn proxydetox_new(
    pac_file: *const libc::c_char,
    #[allow(unused_variables)] negotiate: bool,
    port: u16,
) -> *mut Server {
    let pac_file = unsafe { CStr::from_ptr(pac_file) };
    let pac_file = pac_file.to_str().unwrap_or_default().to_owned();

    let pac_script = load_pac_file(&pac_file);

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

    let server = unsafe { &mut *server };

    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("proxydetox-tokio-rt")
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async move {
        let _ = server.run().await;
    });
}

#[no_mangle]
pub extern "C" fn proxydetox_shutdown(server: *mut Server) {
    use tokio::runtime::Runtime;

    if server != null_mut() {
        let server = unsafe { &mut *server };

        let tx = server.control_channel();
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let _ = tx.send(proxydetox::Command::Shutdown).await;
        });
    }
}

#[no_mangle]
pub extern "C" fn proxydetox_drop(server: *mut Server) {
    if server != null_mut() {
        unsafe { Box::from_raw(server) };
    }
}
