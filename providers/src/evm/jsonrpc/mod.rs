use crate::{
    evm::{
        jsonrpc::{contract::*, types::RpcResponse},
        EvmChain,
    },
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

async fn get_coin_balance(chain: EvmChain, address: Address) -> Result<U256, RpcError> {
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
        Ok(
            futures::future::join_all(addresses.iter().map(|address| async {
                self.get_balance_for_one(chain, token_type, *address)
                    .await
                    .unwrap_or(U256::from(0))
            }))
            .await,
        )
    }

    async fn get_balance_for_one(
        &self,
        chain: EvmChain,
        token_type: TokenType,
        user_address: Self::Address,
    ) -> Result<U256, Self::Error> {
        match token_type {
            TokenType::Coin => get_coin_balance(chain, user_address).await,
            TokenType::Fungible { address } => {
                get_erc20_balance(chain, address, user_address).await
            }
            TokenType::NonFungible { address, id } => {
                get_erc721_balance(chain, address, id, user_address).await
            }
            TokenType::Special { address, id } => {
                get_erc1155_balance(chain, address, id, user_address).await
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::evm::{
        jsonrpc::{
            contract::{get_erc1155_balance, get_erc20_balance, get_erc721_balance},
            get_coin_balance,
        },
        EvmChain,
    };
    use ethereum_types::U256;
    use rusty_gate_common::address;

    #[tokio::test]
    async fn rpc_get_coin_balance() {
        assert!(get_coin_balance(
            EvmChain::Ethereum,
            address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn rpc_get_erc20_balance() {
        assert_eq!(
            get_erc20_balance(
                EvmChain::Ethereum,
                address!("0x458691c1692cd82facfb2c5127e36d63213448a8"),
                address!("0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3")
            )
            .await
            .unwrap(),
            U256::from(100000000000000000000_u128)
        );
    }

    #[tokio::test]
    async fn rpc_get_erc721_balance() {
        assert_eq!(
            get_erc721_balance(
                EvmChain::Ethereum,
                address!("0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85"),
                None,
                address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")
            )
            .await
            .unwrap(),
            U256::from(1)
        );
    }

    #[tokio::test]
    async fn rpc_get_erc1155_balance() {
        assert_eq!(
            get_erc1155_balance(
                EvmChain::Ethereum,
                address!("0x76be3b62873462d2142405439777e971754e8e77"),
                Some(U256::from(10868)),
                address!("0x283d678711daa088640c86a1ad3f12c00ec1252e")
            )
            .await
            .unwrap(),
            U256::from(16)
        );
    }
}
