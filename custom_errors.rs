use thiserror::Error;
use std::fmt;
use std::num::ParseIntError;
use aqueiralibrary::errors::Errors;
use log::{error};
use tokio::sync::AcquireError;


#[derive(Debug, Error)]
pub enum Error {
    #[error("Error: {0}")]
    Error(String),
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("ToStr error: {0}")]
    ToStrError(#[from] hyper::header::ToStrError),
    #[error("Parse int error: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("AcquireError: {0}")]
    AcquireError(#[from] AcquireError),
    #[error("std::io::Error: {0}")]
    STDIOError(#[from] std::io::Error),
    #[error("aqueiralibraryerror: {0}")]
    AqueiraErrors(#[from] Errors)
}
impl Error {
    pub fn any(message: &str, err: impl fmt::Display) -> Self {
        error!("{} -> {}", message, err);
        Error::Error(format!("{} -> {}", message, err))
    }
}
impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error::Error("An unexpected error occurred".to_string())
    }
}



