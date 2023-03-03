use crate::{
    evm::{balancy::BalancyProvider, get_provider, jsonrpc::contract::*, ProviderConfigError},
    BalanceQuerier, TokenType,
};
use async_trait::async_trait;
pub use contract::get_erc20_decimals;
use futures::future::join_all;
use guild_common::Scalar;
use primitive_types::{H160 as Address, U256};
use serde::Deserialize;
use serde_json::{json, Value};
use std::str::FromStr;
use thiserror::Error;

mod contract;

const ETH_BALANCE_DIVIDER: Scalar = 10_u128.pow(18) as Scalar;

#[derive(Deserialize)]
pub struct RpcResponse {
    pub result: String,
}

pub struct RpcProvider;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Chain `{0}` is not supported")]
    ChainNotSupported(String),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Config(#[from] ProviderConfigError),
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

async fn get_coin_balance(
    client: &reqwest::Client,
    chain: &str,
    address: Address,
) -> Result<Scalar, RpcError> {
    let provider = get_provider(chain)?;

    let payload = create_payload("eth_getBalance", json!([address, "latest"]), 1);

    let res: RpcResponse = client
        .post(&provider.rpc_url)
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    let balance = rpc_error!(U256::from_str(&res.result))?;

    Ok((balance.as_u128() as f64) / ETH_BALANCE_DIVIDER)
}

#[async_trait]
impl BalanceQuerier for RpcProvider {
    type Error = RpcError;
    type Address = Address;
    type Id = U256;

    async fn get_balance(
        &self,
        client: &reqwest::Client,
        chain: &str,
        token_type: TokenType<Self::Address, Self::Id>,
        user_address: Self::Address,
    ) -> Result<Scalar, Self::Error> {
        match token_type {
            TokenType::Native => get_coin_balance(client, chain, user_address).await,
            TokenType::Fungible { address } => {
                get_erc20_balance(client, chain, address, user_address).await
            }
            TokenType::NonFungible { address, id } => {
                get_erc721_balance(client, chain, address, id, user_address).await
            }
            TokenType::Special { address, id } => match id {
                Some(token_id) => {
                    get_erc1155_balance(client, chain, address, token_id, user_address).await
                }
                None => rpc_error!(
                    BalancyProvider
                        .get_balance(client, chain, token_type, user_address)
                        .await
                ),
            },
        }
    }

    async fn get_balance_batch(
        &self,
        client: &reqwest::Client,
        chain: &str,
        token_type: TokenType<Self::Address, Self::Id>,
        addresses: &[Self::Address],
    ) -> Result<Vec<Scalar>, Self::Error> {
        match token_type {
            TokenType::Native => get_eth_balance_batch(client, chain, addresses).await,
            TokenType::Fungible { address } => {
                get_erc20_balance_batch(client, chain, address, addresses).await
            }
            TokenType::NonFungible { address, id: _ } => {
                get_erc721_balance_batch(client, chain, address, addresses).await
            }
            TokenType::Special { address, id } => match id {
                Some(token_id) => {
                    get_erc1155_balance_batch(client, chain, address, token_id, addresses).await
                }
                None => {
                    let res = join_all(addresses.iter().map(|addr| async {
                        rpc_error!(
                            BalancyProvider
                                .get_balance(client, chain, token_type, *addr)
                                .await
                        )
                    }))
                    .await;

                    res.into_iter().collect()
                }
            },
        }
    }
}

#[cfg(all(test, feature = "nomock"))]
mod test {
    use crate::{
        evm::{common::*, jsonrpc::RpcProvider},
        BalanceQuerier,
    };
    use guild_common::{address, Chain::Ethereum, TokenType::*};
    use primitive_types::U256;

    #[tokio::test]
    async fn rpc_get_coin_balance() {
        assert_eq!(
            RpcProvider
                .get_balance(
                    &reqwest::Client::new(),
                    &Ethereum.to_string(),
                    Native,
                    address!(USER_1_ADDR)
                )
                .await
                .unwrap(),
            0.000464468855704627
        );
    }

    #[tokio::test]
    async fn rpc_get_coin_balance_batch() {
        assert_eq!(
            RpcProvider
                .get_balance_batch(
                    &reqwest::Client::new(),
                    &Ethereum.to_string(),
                    Native,
                    &vec![address!(USER_1_ADDR), address!(USER_2_ADDR)]
                )
                .await
                .unwrap(),
            vec![0.000464468855704627, 0.3919455024496939]
        );
    }

    #[tokio::test]
    async fn rpc_get_erc20_balance() {
        let token_type = Fungible {
            address: address!(ERC20_ADDR),
        };

        assert_eq!(
            RpcProvider
                .get_balance(
                    &reqwest::Client::new(),
                    &Ethereum.to_string(),
                    token_type,
                    address!(USER_2_ADDR)
                )
                .await
                .unwrap(),
            100.0
        );
    }

    #[tokio::test]
    async fn rpc_get_erc20_balance_batch() {
        let token_type = Fungible {
            address: address!(ERC20_ADDR),
        };

        assert_eq!(
            RpcProvider
                .get_balance_batch(
                    &reqwest::Client::new(),
                    &Ethereum.to_string(),
                    token_type,
                    &vec![address!(USER_1_ADDR), address!(USER_2_ADDR)]
                )
                .await
                .unwrap(),
            vec![0.0, 100.0]
        );
    }

    #[tokio::test]
    async fn rpc_get_erc721_balance() {
        let client = reqwest::Client::new();
        let chain = Ethereum.to_string();
        let token_type_without_id = NonFungible {
            address: address!(ERC721_ADDR),
            id: None,
        };
        let token_type_with_id = NonFungible {
            address: address!(ERC721_ADDR),
            id: Some(U256::from_dec_str(ERC721_ID).unwrap()),
        };
        let user_address = address!(USER_1_ADDR);

        assert_eq!(
            RpcProvider
                .get_balance(&client, &chain, token_type_without_id, user_address)
                .await
                .unwrap(),
            1.0
        );
        assert_eq!(
            RpcProvider
                .get_balance(&client, &chain, token_type_with_id, user_address)
                .await
                .unwrap(),
            1.0
        );
    }

    #[tokio::test]
    async fn rpc_get_erc721_balance_batch() {
        let client = reqwest::Client::new();
        let token_type_without_id = NonFungible {
            address: address!(ERC721_ADDR),
            id: None,
        };

        assert_eq!(
            RpcProvider
                .get_balance_batch(
                    &client,
                    &Ethereum.to_string(),
                    token_type_without_id,
                    &vec![address!(USER_1_ADDR), address!(USER_2_ADDR)]
                )
                .await
                .unwrap(),
            vec![1.0, 1.0]
        );
    }

    #[tokio::test]
    async fn rpc_get_erc1155_balance() {
        let client = reqwest::Client::new();
        let chain = Ethereum.to_string();
        let token_type_without_id = Special {
            address: address!(ERC1155_ADDR),
            id: None,
        };
        let token_type_with_id = Special {
            address: address!(ERC1155_ADDR),
            id: Some(U256::from(ERC1155_ID)),
        };
        let user_address = address!(USER_3_ADDR);

        assert!(
            RpcProvider
                .get_balance(&client, &chain, token_type_without_id, user_address)
                .await
                .unwrap()
                > 6000.0
        );
        assert_eq!(
            RpcProvider
                .get_balance(&client, &chain, token_type_with_id, user_address)
                .await
                .unwrap(),
            16.0
        );
    }

    #[tokio::test]
    async fn rpc_get_erc1155_balance_batch() {
        let client = reqwest::Client::new();
        let token_type_with_id = Special {
            address: address!(ERC1155_ADDR),
            id: Some(U256::from(ERC1155_ID)),
        };

        assert_eq!(
            RpcProvider
                .get_balance_batch(
                    &client,
                    &Ethereum.to_string(),
                    token_type_with_id,
                    &vec![address!(USER_1_ADDR), address!(USER_3_ADDR)]
                )
                .await
                .unwrap(),
            vec![0.0, 16.0]
        );
    }
}
