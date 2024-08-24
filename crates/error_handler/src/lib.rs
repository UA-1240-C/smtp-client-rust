use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Tls(tokio_native_tls::native_tls::Error),
    Smtp(String),
    Unwrap(String),
    TlsUpgrade(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<tokio_native_tls::native_tls::Error> for Error {
    fn from(err: tokio_native_tls::native_tls::Error) -> Self {
        Error::Tls(err)
    }
}