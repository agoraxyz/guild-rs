use crate::evm::jsonrpc::RpcError;
use primitive_types::{H160 as Address, U256};
use serde::{de::Error, Deserialize, Deserializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BalancyError {
    #[error("Chain `{0}` is not supported by Balancy")]
    ChainNotSupported(String),
    #[error("Unsupported token type")]
    TokenTypeNotSupported(String),
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
pub struct Erc20 {
    pub token_address: Address,
    #[serde(deserialize_with = "u256_from_str")]
    pub amount: U256,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Erc721 {
    pub token_address: Address,
    #[serde(deserialize_with = "u256_from_str")]
    pub token_id: U256,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Erc1155 {
    pub token_address: Address,
    #[serde(deserialize_with = "u256_from_str")]
    pub token_id: U256,
    #[serde(deserialize_with = "u256_from_str")]
    pub amount: U256,
}

#[derive(Deserialize, Debug)]
pub struct BalancyResponse<T> {
    pub result: Vec<T>,
}
