use proxydetox::auth::netrc;
use proxydetox::auth::AuthenticatorFactory;
use proxydetox::http_file;
use std::ffi::CStr;
use std::fs::read_to_string;
use std::fs::File;
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::sync::oneshot;

type ServerFuture = dyn Future<Output = std::result::Result<(), hyper::Error>>;

pub struct Server {
    server: Option<Pin<Box<ServerFuture>>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

fn netrc_path() -> PathBuf {
    let mut netrc_path = dirs::home_dir().unwrap_or_default();
    netrc_path.push(".netrc");
    netrc_path
}

fn load_netrc_store() -> netrc::Store {
    File::open(netrc_path())
        .ok()
        .and_then(|f| netrc::Store::new(std::io::BufReader::new(f)).ok())
        .unwrap_or_default()
}

fn load_pac_file(path: &str) -> Option<String> {
    use tokio::runtime::Runtime;

    let content = if path.starts_with("http://") {
        if let Ok(url) = path.parse() {
            let rt = Runtime::new().unwrap();
            rt.block_on(async { http_file(url).await.unwrap_or_default() })
        } else {
            "".into()
        }
    } else {
        read_to_string(path).unwrap_or_default()
    };

    if content.trim().is_empty() {
        None
    } else {
        Some(content)
    }
}

/// # Safety
/// Caller must ensure `server` is valid.
#[no_mangle]
pub unsafe extern "C" fn proxydetox_new(
    pac_file: *const libc::c_char,
    #[allow(unused_variables)] negotiate: bool,
    port: u16,
) -> *mut Server {
    let pac_file = CStr::from_ptr(pac_file);
    let pac_file = pac_file.to_str().unwrap_or_default().to_owned();

    let pac_script = load_pac_file(&pac_file);

    #[cfg(feature = "negotiate")]
    let auth = if negotiate {
        #[cfg(feature = "negotiate")]
        AuthenticatorFactory::negotiate(Vec::new())
    } else {
        AuthenticatorFactory::basic(load_netrc_store())
    };
    #[cfg(not(feature = "negotiate"))]
    let auth = AuthenticatorFactory::basic(load_netrc_store());

    let session = proxydetox::Session::builder()
        .pac_script(pac_script)
        .authenticator_factory(Some(auth))
        .build();

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let server = hyper::Server::bind(&addr).serve(session);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = server.with_graceful_shutdown(async {
        shutdown_rx.await.ok();
    });
    let server = Some(Box::pin(server) as Pin<Box<dyn Future<Output = Result<(), hyper::Error>>>>);

    let server = Box::new(Server {
        server,
        shutdown_tx: Some(shutdown_tx),
    });

    Box::<Server>::into_raw(server)
}

/// # Safety
/// Caller must ensure `server` is valid.
#[no_mangle]
pub unsafe extern "C" fn proxydetox_run(server: *mut Server) {
    use tokio::runtime::Builder;

    let server = &mut *server;

    if let Some(mut server) = server.server.take() {
        let runtime = Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("proxydetox-tokio-rt")
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async move {
            let server = std::pin::Pin::new(&mut server).get_mut();
            let _ = server.await;
        });
    }
}

/// Will stop the proxydetox server and releases the memory of the server struct.
/// # Safety
/// Caller must ensure `server` is valid.
#[no_mangle]
pub unsafe extern "C" fn proxydetox_shutdown(server: *mut Server) {
    if !server.is_null() {
        let server = Box::from_raw(server);
        server.shutdown_tx.map(|x| x.send(()).ok());
    }
}
