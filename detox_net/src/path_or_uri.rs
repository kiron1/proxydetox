use std::{fmt::Display, path::PathBuf, str::FromStr};

use http::Uri;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathOrUri {
    Path(PathBuf),
    Uri(Uri),
}

impl Display for PathOrUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(p) => p.display().fmt(f),
            Self::Uri(u) => u.fmt(f),
        }
    }
}

impl FromStr for PathOrUri {
    type Err = http::uri::InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http://") || s.starts_with("https://") {
            Ok(Self::from(s.parse::<Uri>()?))
        } else {
            Ok(Self::from(PathBuf::from(s)))
        }
    }
}

impl From<PathBuf> for PathOrUri {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
    }
}

impl From<Uri> for PathOrUri {
    fn from(uri: Uri) -> Self {
        Self::Uri(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::PathOrUri;
    use http::Uri;
    use std::path::PathBuf;

    #[test]
    fn path_or_uri_test() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            PathOrUri::Path(PathBuf::from("README.md")),
            "README.md".parse::<PathOrUri>()?
        );
        assert_eq!(
            PathOrUri::Path(PathBuf::from("/etc/motd")),
            "/etc/motd".parse::<PathOrUri>()?
        );
        assert_eq!(
            PathOrUri::Uri("http://example.org/index.html".parse::<Uri>()?),
            "http://example.org/index.html".parse::<PathOrUri>()?
        );
        assert_eq!(
            PathOrUri::Uri("https://example.org/index.html".parse::<Uri>()?),
            "https://example.org/index.html".parse::<PathOrUri>()?
        );
        Ok(())
    }
}
