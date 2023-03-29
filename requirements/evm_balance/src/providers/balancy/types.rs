use crate::providers::jsonrpc::RpcError;
use primitive_types::U256;
use serde::{de::Error, Deserialize, Deserializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BalancyError {
    #[error("Invalid Balancy request")]
    InvalidBalancyRequest,
    #[error("Too many requests to Balancy")]
    TooManyRequests,
    #[error(transparent)]
    Rpc(#[from] RpcError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("Got response with status code `{0}`")]
    Unknown(u16),
}

fn u256_from_str<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    U256::from_dec_str(s).map_err(D::Error::custom)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Erc1155 {
    pub token_address: String,
    pub token_id: String,
    #[serde(deserialize_with = "u256_from_str")]
    pub amount: U256,
}

#[derive(Deserialize, Debug)]
pub struct BalancyResponse {
    pub result: Vec<Erc1155>,
}
