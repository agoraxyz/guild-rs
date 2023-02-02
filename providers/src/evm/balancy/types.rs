use crate::evm::u256_from_str;
use ethereum_types::{Address, U256};
use serde::Deserialize;
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
    Reqwest(#[from] reqwest::Error),
    #[error("Got response with status code `{0}`")]
    Unknown(u16),
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
