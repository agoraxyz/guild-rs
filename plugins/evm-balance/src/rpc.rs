use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
pub struct RpcResponse {
    pub result: String,
}

#[derive(Error, Debug)]
pub enum RpcError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Other(String),
}

#[macro_export]
macro_rules! rpc_error {
    ($code:expr) => {
        $code.map_err(|err| RpcError::Other(err.to_string()))
    };
}
