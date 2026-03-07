use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum MempoolError {
    Mdbx(String),
    Io(std::io::Error),
}

impl Display for MempoolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mdbx(err) => write!(f, "MDBX error: {err}"),
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

impl From<reth_libmdbx::Error> for MempoolError {
    fn from(value: reth_libmdbx::Error) -> Self {
        Self::Mdbx(value.to_string())
    }
}
