#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypeError {
    NoneType,
    UnknownType,
    BadString,
}

impl std::error::Error for TypeError {}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match *self {
            TypeError::NoneType => write!(f, "none type error"),
            TypeError::UnknownType => write!(f, "unknown type error"),
            TypeError::BadString => write!(f, "bad string error"),
        }
    }
}
