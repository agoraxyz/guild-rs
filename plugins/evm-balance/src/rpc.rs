use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
pub struct Response {
    pub result: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Other(String),
}

#[macro_export]
macro_rules! rpc_error {
    ($code:expr) => {
        $code.map_err(|err| Error::Other(err.to_string()))
    };
}
