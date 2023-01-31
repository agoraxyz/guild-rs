use crate::{
    evm::{jsonrpc::types::RpcResponse, EvmChain},
    BalanceQuerier, TokenType, CLIENT,
};
use async_trait::async_trait;
use ethereum_types::{Address, U256};
use std::fmt;
use thiserror::Error;

mod contract;
mod types;

pub struct RpcProvider;

pub const RPC_PROVIDER: RpcProvider = RpcProvider {};

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Chain `{0}` is not supported")]
    ChainNotSupported(String),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("Got response with status code `{0}`")]
    Unknown(u16),
}

enum JsonRpcMethods {
    EthGetBalance,
    EthCall,
}

impl fmt::Display for JsonRpcMethods {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                JsonRpcMethods::EthGetBalance => "eth_getBalance",
                JsonRpcMethods::EthCall => "eth_call",
            }
        )
    }
}

fn create_payload(method: JsonRpcMethods, params: String, id: u32) -> String {
    format!(
        "{{
            \"method\"  : \"{method}\",
            \"params\"  : {params},
            \"id\"      : {id},
            \"jsonrpc\" : \"2.0\"
        }}"
    )
}

const ETHEREUM: &str = "https://mainnet.infura.io/v3/";

async fn get_coin_balance(address: Address) -> Result<U256, RpcError> {
    let payload = create_payload(
        JsonRpcMethods::EthGetBalance,
        format!("[\"{address:?}\", \"latest\"]"),
        1,
    );

    let res: RpcResponse = CLIENT
        .read()
        .await
        .post(ETHEREUM)
        .body(payload)
        .send()
        .await?
        .json()
        .await?;

    Ok(res.result)
}

#[async_trait]
impl BalanceQuerier for RpcProvider {
    type Error = RpcError;
    type Address = Address;

    async fn get_balance_for_many(
        &self,
        chain: EvmChain,
        token_type: TokenType,
        addresses: &[Self::Address],
    ) -> Result<Vec<U256>, Self::Error> {
        todo!()
    }

    async fn get_balance_for_one(
        &self,
        chain: EvmChain,
        token_type: TokenType,
        user_address: Self::Address,
    ) -> Result<U256, Self::Error> {
        match token_type {
            TokenType::Coin => get_coin_balance(user_address).await,
            TokenType::Fungible { address } => contract::call_contract(address, user_address).await,
            TokenType::NonFungible { address, id } => todo!(),
            TokenType::Special { address, id } => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::get_coin_balance;
    use rusty_gate_common::address;

    #[tokio::test]
    async fn rpc_test() {
        assert!(
            get_coin_balance(address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"))
                .await
                .is_ok()
        );
    }
}
