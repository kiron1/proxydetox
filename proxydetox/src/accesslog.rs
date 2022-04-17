use std::{fmt::Write, net::SocketAddr};

use chrono::{DateTime, Duration, Local, SecondsFormat};

use http::StatusCode;
use paclib::ProxyDesc;

#[derive(Clone, Debug)]
enum Response {
    Success {
        status_code: http::StatusCode,
        bytes: Option<u64>,
    },
    Error(String),
}

#[derive(Clone, Debug)]
pub struct Entry {
    timestamp: DateTime<Local>,
    peer_addr: SocketAddr,
    method: http::Method,
    uri: http::Uri,
    version: http::Version,
    user_agent: Option<String>,
    proxy: ProxyDesc,
    response: Response,
    duration: Duration,
}

pub struct EntryBegin {
    timestamp: DateTime<Local>,
    peer_addr: SocketAddr,
    method: http::Method,
    uri: http::Uri,
    version: http::Version,
    user_agent: Option<String>,
}

impl EntryBegin {
    pub fn success(self, proxy: ProxyDesc, status_code: StatusCode, bytes: Option<u64>) -> Entry {
        Entry {
            timestamp: self.timestamp,
            peer_addr: self.peer_addr,
            method: self.method,
            uri: self.uri,
            version: self.version,
            user_agent: self.user_agent,
            proxy,
            response: Response::Success { status_code, bytes },
            duration: Local::now() - self.timestamp,
        }
    }

    pub fn error(self, proxy: ProxyDesc, error: &impl std::error::Error) -> Entry {
        Entry {
            timestamp: self.timestamp,
            peer_addr: self.peer_addr,
            method: self.method,
            uri: self.uri,
            version: self.version,
            user_agent: self.user_agent,
            proxy,
            response: Response::Error(error.to_string()),
            duration: Local::now() - self.timestamp,
        }
    }
}

impl Entry {
    pub fn begin(
        peer_addr: SocketAddr,
        method: http::Method,
        uri: http::Uri,
        version: http::Version,
        user_agent: Option<String>,
    ) -> EntryBegin {
        EntryBegin {
            timestamp: Local::now(),
            peer_addr,
            method,
            uri,
            version,
            user_agent,
        }
    }
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} \"{}\" \"{} {} {:?}\" {:.3}s",
            self.timestamp.to_rfc3339_opts(SecondsFormat::Secs, false),
            self.peer_addr,
            self.proxy,
            self.method,
            self.uri,
            self.version,
            self.duration.num_milliseconds() as f32 * 1e-6,
        )?;
        match self.response {
            Response::Success { status_code, bytes } => {
                write!(f, " {}", status_code)?;
                if let Some(bytes) = bytes {
                    write!(f, " {}b", bytes)?;
                } else {
                    f.write_str(" -")?;
                }
            }
            Response::Error(ref cause) => write!(f, " error: \"{}\"", cause)?,
        }
        if let Some(ref ua) = self.user_agent {
            f.write_str(" \"")?;
            f.write_str(ua)?;
            f.write_char('"')?;
        } else {
            f.write_str(" -")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use paclib::ProxyDesc;

    use super::Entry;

    #[test]
    fn test_success_entry() {
        let entry = Entry::begin(
            "127.0.0.1:34524".parse().unwrap(),
            http::Method::GET,
            "http://localhost:8080".parse().unwrap(),
            http::Version::HTTP_11,
            Some("curl/7.79.1".to_string()),
        );
        let entry = entry.success(
            ProxyDesc::Proxy("127.0.0.1:8080".parse().unwrap()),
            http::StatusCode::OK,
            Some(4096),
        );
        let entry = entry.to_string();

        assert!(entry.contains("127.0.0.1:34524"));
        assert!(entry.contains("127.0.0.1:8080"));
        assert!(entry.contains("GET"));
        assert!(entry.contains("localhost:8080"));
        assert!(entry.contains("HTTP/1.1"));
        assert!(entry.contains("\"curl/7.79.1\""));
        assert!(entry.contains("OK"));
        assert!(entry.contains("4096b"));
        assert!(!entry.contains(" - "));
    }

    #[test]
    fn test_success_without_size_entry() {
        let entry = Entry::begin(
            "127.0.0.1:34524".parse().unwrap(),
            http::Method::GET,
            "http://localhost:8080".parse().unwrap(),
            http::Version::HTTP_11,
            Some("curl/7.79.1".to_string()),
        );
        let entry = entry.success(ProxyDesc::Direct, http::StatusCode::OK, None);
        let entry = entry.to_string();

        assert!(entry.contains(" - "));
    }

    #[test]
    fn test_success_without_agent_entry() {
        let entry = Entry::begin(
            "127.0.0.1:34524".parse().unwrap(),
            http::Method::GET,
            "http://localhost:8080".parse().unwrap(),
            http::Version::HTTP_11,
            None,
        );
        let entry = entry.success(ProxyDesc::Direct, http::StatusCode::OK, Some(1024));
        let entry = entry.to_string();

        assert!(entry.contains(" -"));
    }

    #[test]
    fn test_error_entry() {
        let entry = Entry::begin(
            "127.0.0.1:34524".parse().unwrap(),
            http::Method::GET,
            "http://localhost:8080".parse().unwrap(),
            http::Version::HTTP_11,
            Some("curl/7.79.1".to_string()),
        );
        let entry = entry.error(
            ProxyDesc::Direct,
            &std::io::Error::new(std::io::ErrorKind::Other, "ERROR"),
        );
        let entry = entry.to_string();

        assert!(entry.contains("127.0.0.1:34524"));
        assert!(entry.contains("DIRECT"));
        assert!(entry.contains("GET"));
        assert!(entry.contains("http://localhost:8080"));
        assert!(entry.contains("HTTP/1.1"));
        assert!(entry.contains("ERROR"));
        assert!(entry.contains("\"curl/7.79.1\""));
    }
}
