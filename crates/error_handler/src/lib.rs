use std::{fmt::Display, net::AddrParseError};
use tokio::time::error::Elapsed;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Tls(tokio_native_tls::native_tls::Error),
    TlsUpgrade(String),
    AddrParseError(std::net::AddrParseError),
    AsyncStream(String),
    ClosedConnection(String),
    SmtpResponse(String),
    MessageBuild(String),
    Timeout(String),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::Io(_), Error::Io(_)) => false, // std::io::Error does not implement PartialEq
            (Error::Tls(_), Error::Tls(_)) => false, // tokio_native_tls::native_tls::Error does not implement PartialEq
            (Error::TlsUpgrade(a), Error::TlsUpgrade(b)) => a == b,
            (Error::AddrParseError(a), Error::AddrParseError(b)) => a == b,
            (Error::AsyncStream(a), Error::AsyncStream(b)) => a == b,
            (Error::ClosedConnection(a), Error::ClosedConnection(b)) => a == b,
            (Error::SmtpResponse(a), Error::SmtpResponse(b)) => a == b,
            (Error::MessageBuild(a), Error::MessageBuild(b)) => a == b,
            (Error::Timeout(a), Error::Timeout(b)) => a == b,
            _ => false,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Error::ClosedConnection(ref msg) = self {
            write!(f, "Connection was closed on try to: {}\n", msg)
        } else {
            write!(f, "{:?}\n", self)
        }
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

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Self {
        Error::AddrParseError(err)
    }
}

impl From<Elapsed> for Error {
    fn from(err: Elapsed) -> Self {
        Error::Timeout(err.to_string())
    }
}