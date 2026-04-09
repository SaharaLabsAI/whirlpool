use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum MempoolError {
    Storage(String),
    Io(std::io::Error),
}

impl Display for MempoolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(err) => write!(f, "Storage error: {err}"),
            Self::Io(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl std::error::Error for MempoolError {}

impl From<std::io::Error> for MempoolError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
