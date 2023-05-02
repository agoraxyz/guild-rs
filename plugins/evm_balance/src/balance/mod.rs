use crate::balance::contract::*;
use guild_common::{Scalar, TokenType};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use thiserror::Error;

mod contract;

#[derive(Clone, Debug, Deserialize)]
pub struct EvmProvider {
    pub rpc_url: String,
    pub contract: String,
}

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

const ETH_BALANCE_DIVIDER: Scalar = 10_u128.pow(18) as Scalar;

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
        addresses: &[&str],
    ) -> Result<Vec<Scalar>, RpcError> {
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
            TokenType::NonFungible { address, id } => {
                get_erc721_balance_batch(
                    client,
                    &self.contract.to_string(),
                    &self.rpc_url,
                    &address,
                    &id,
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
mod common {
    pub const RPC_URL: &str = "https://eth.public-rpc.com";
    pub const USER_1_ADDR: &str = "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE";
    pub const USER_2_ADDR: &str = "0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3";
    pub const USER_3_ADDR: &str = "0x283D678711dAa088640C86a1ad3f12C00EC1252E";
    pub const ERC20_ADDR: &str = "0x458691c1692CD82faCfb2C5127e36D63213448A8";
    pub const ERC721_ADDR: &str = "0x57f1887a8BF19b14fC0dF6Fd9B2acc9Af147eA85";
    pub const ERC721_ID: &str =
        "61313325075603536901663283754390960556726744542208800735045237225934362163454";
    pub const ERC1155_ADDR: &str = "0x76BE3b62873462d2142405439777e971754E8E77";
    pub const ERC1155_ID: usize = 10868;
}

#[cfg(test)]
mod test {
    use crate::balance::{common::*, EvmProvider};
    use guild_common::TokenType::*;
    use primitive_types::U256;
    use reqwest::Client;

    fn provider() -> EvmProvider {
        EvmProvider {
            rpc_url: RPC_URL.to_string(),
            contract: "0x5BA1e12693Dc8F9c48aAD8770482f4739bEeD696".to_string(),
        }
    }

    #[tokio::test]
    async fn rpc_get_coin_balance_batch() {
        let client: &'static Client = Box::leak(Box::new(Client::new()));

        assert_eq!(
            provider()
                .get_balance_batch(client, Native, &[USER_1_ADDR, USER_2_ADDR])
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
                .get_balance_batch(client, token_type, &[USER_1_ADDR, USER_2_ADDR])
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
                .get_balance_batch(client, token_type_without_id, &[USER_1_ADDR, USER_2_ADDR])
                .await
                .unwrap(),
            vec![1.0, 1.0]
        );
        assert_eq!(
            provider()
                .get_balance_batch(client, token_type_with_id, &[USER_1_ADDR, USER_2_ADDR])
                .await
                .unwrap(),
            vec![1.0, 0.0]
        );
    }

    #[tokio::test]
    async fn rpc_get_erc1155_balance_batch() {
        let client: &'static Client = Box::leak(Box::new(Client::new()));

        let token_type_with_id = Special {
            address: ERC1155_ADDR.to_string(),
            id: Some(U256::from(ERC1155_ID).to_string()),
        };

        assert_eq!(
            provider()
                .get_balance_batch(client, token_type_with_id, &[USER_1_ADDR, USER_3_ADDR])
                .await
                .unwrap(),
            vec![0.0, 15.0]
        );
    }
}
