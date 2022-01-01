use proxydetox::auth::AuthenticatorFactory;
use proxydetox::detox::Config;
use proxydetox::detox::Session;
use proxydetox::http_file;
use proxydetox::Server;
use std::ffi::CStr;
use std::fs::read_to_string;
use std::path::PathBuf;

fn netrc_path() -> PathBuf {
    let mut netrc_path = dirs::home_dir().unwrap_or_default();
    netrc_path.push(".netrc");
    netrc_path
}

fn load_pac_file(path: &str) -> String {
    use tokio::runtime::Runtime;

    let content = if path.starts_with("http://") {
        if let Ok(url) = path.parse() {
            let rt = Runtime::new().unwrap();
            rt.block_on(async { http_file(url).await.unwrap_or_default() })
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

pub struct ServerSession {
    server: Server,
    session: Session,
}

/// # Safety
/// Caller must ensure `server` is valid.
#[no_mangle]
pub unsafe extern "C" fn proxydetox_new(
    pac_file: *const libc::c_char,
    #[allow(unused_variables)] negotiate: bool,
    port: u16,
) -> *mut ServerSession {
    let pac_file = CStr::from_ptr(pac_file);
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

    let server = Server::new(port);
    let session = Session::new(&pac_script, auth, config);

    let server_session = Box::new(ServerSession { server, session });
    Box::<ServerSession>::into_raw(server_session)
}

/// # Safety
/// Caller must ensure `server` is valid.
#[no_mangle]
pub unsafe extern "C" fn proxydetox_run(server_session: *mut ServerSession) {
    use tokio::runtime::Builder;

    let server_session = &mut *server_session;

    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("proxydetox-tokio-rt")
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async move {
        let _ = server_session
            .server
            .run(server_session.session.clone())
            .await;
    });
}

/// # Safety
/// Caller must ensure `server` is valid.
#[no_mangle]
pub unsafe extern "C" fn proxydetox_shutdown(server_session: *mut ServerSession) {
    use tokio::runtime::Runtime;

    if !server_session.is_null() {
        let server_session = &mut *server_session;

        let tx = server_session.server.control_channel();
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let _ = tx.send(proxydetox::Command::Shutdown).await;
        });
    }
}

/// # Safety
/// Caller must ensure `server` is valid.
#[no_mangle]
pub unsafe extern "C" fn proxydetox_drop(server_session: *mut ServerSession) {
    if !server_session.is_null() {
        Box::from_raw(server_session);
    }
}
