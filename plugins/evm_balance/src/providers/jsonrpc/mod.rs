use crate::providers::{
    jsonrpc::contract::*, EvmProvider, TokenType,
};
pub use contract::get_erc20_decimals;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use thiserror::Error;

mod contract;

const ETH_BALANCE_DIVIDER: f64 = 10_u128.pow(18) as f64;

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

fn create_payload(method: &str, params: Value, id: u32) -> Value {
    json!({
        "method"  : method,
        "params"  : params,
        "id"      : id,
        "jsonrpc" : "2.0"
    })
}

impl EvmProvider {
    pub async fn get_balance_batch(
        &self,
        client: &'static Client,
        token_type: TokenType,
        addresses: &[String],
    ) -> Result<Vec<f64>, RpcError> {
        match token_type {
            TokenType::Native => {
                get_eth_balance_batch(client, &self.contract.to_string(), &self.rpc_url, addresses)
                    .await
            }
            TokenType::Fungible { address } => {
                get_erc20_balance_batch(
                    client,
                    &self.contract.to_string(),
                    &self.rpc_url,
                    &address,
                    addresses,
                )
                .await
            }
            TokenType::NonFungible { address, id: _ } => {
                get_erc721_balance_batch(
                    client,
                    &self.contract.to_string(),
                    &self.rpc_url,
                    &address,
                    addresses,
                )
                .await
            }
            TokenType::Special { address, id } => match id {
                Some(token_id) => {
                    get_erc1155_balance_batch(
                        client,
                        &self.rpc_url,
                        address.clone(),
                        &token_id,
                        addresses,
                    )
                    .await
                }
                None => Ok(vec![0.0; addresses.len()]),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::providers::{common::*, EvmProvider};
    use guild_common::TokenType::*;
    use primitive_types::U256;
    use reqwest::Client;

    fn provider() -> EvmProvider {
        EvmProvider {
            rpc_url: RPC_URL.to_string(),
            contract: "0x5ba1e12693dc8f9c48aad8770482f4739beed696".to_string(),
        }
    }

    #[tokio::test]
    async fn rpc_get_coin_balance_batch() {
        let client: &'static Client = Box::leak(Box::new(Client::new()));

        assert_eq!(
            provider()
                .get_balance_batch(
                    client,
                    Native,
                    &[USER_1_ADDR.to_string(), USER_2_ADDR.to_string()]
                )
                .await
                .unwrap(),
            vec![0.000464468855704627, 0.3919455024496939]
        );
    }

    #[tokio::test]
    async fn rpc_get_erc20_balance_batch() {
        let token_type = Fungible {
            address: ERC20_ADDR.to_string(),
        };
        let client: &'static Client = Box::leak(Box::new(Client::new()));

        assert_eq!(
            provider()
                .get_balance_batch(
                    client,
                    token_type,
                    &[USER_1_ADDR.to_string(), USER_2_ADDR.to_string()]
                )
                .await
                .unwrap(),
            vec![0.0, 100.0]
        );
    }

    #[tokio::test]
    async fn rpc_get_erc721_balance_batch() {
        let client: &'static Client = Box::leak(Box::new(Client::new()));

        let token_type_without_id = NonFungible {
            address: ERC721_ADDR.to_string(),
            id: None,
        };
        let token_type_with_id = NonFungible {
            address: ERC721_ADDR.to_string(),
            id: Some(ERC721_ID.to_string()),
        };

        assert_eq!(
            provider()
                .get_balance_batch(
                    client,
                    token_type_without_id,
                    &[USER_1_ADDR.to_string(), USER_2_ADDR.to_string()]
                )
                .await
                .unwrap(),
            vec![1.0, 1.0]
        );
        assert_eq!(
            provider()
                .get_balance_batch(
                    client,
                    token_type_with_id,
                    &[USER_1_ADDR.to_string(), USER_2_ADDR.to_string()]
                )
                .await
                .unwrap(),
            vec![1.0, 1.0]
        );
    }

    #[tokio::test]
    async fn rpc_get_erc1155_balance_batch() {
        let client: &'static Client = Box::leak(Box::new(Client::new()));

        let token_type_without_id = Special {
            address: ERC1155_ADDR.to_string(),
            id: None,
        };
        let token_type_with_id = Special {
            address: ERC1155_ADDR.to_string(),
            id: Some(U256::from(ERC1155_ID).to_string()),
        };

        assert_eq!(
            provider()
                .get_balance_batch(
                    client,
                    token_type_without_id,
                    &[USER_1_ADDR.to_string(), USER_3_ADDR.to_string()]
                )
                .await
                .unwrap(),
            vec![0.0, 6510.0]
        );
        assert_eq!(
            provider()
                .get_balance_batch(
                    client,
                    token_type_with_id,
                    &[USER_1_ADDR.to_string(), USER_3_ADDR.to_string()]
                )
                .await
                .unwrap(),
            vec![0.0, 16.0]
        );
    }
}
