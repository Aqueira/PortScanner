use std::fmt;
use hyper::header::ToStrError;
use tokio::task::JoinError;

#[derive(Debug)]
pub enum Errors {
    Error,
}
impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Errors::Error => write!(f, "Error: "),
        }
    }
}
impl std::error::Error for Errors {}
impl From<JoinError> for Errors {
    fn from(_error: JoinError) -> Self {
        Errors::Error
    }
}
impl From<reqwest::Error> for Errors {
    fn from(_error: reqwest::Error) -> Self {
        Errors::Error
    }
}
impl From<ToStrError> for Errors {
    fn from(_: ToStrError) -> Self {
        Errors::Error
    }
}
