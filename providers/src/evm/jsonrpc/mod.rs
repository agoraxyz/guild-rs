use crate::{
    evm::{balancy::BalancyProvider, jsonrpc::contract::*, EvmChain},
    BalanceQuerier, TokenType,
};
use async_trait::async_trait;
use config::{Config, File};
pub use contract::get_erc20_decimals;
use ethereum_types::{Address, U256};
use futures::future::join_all;
use serde::Deserialize;
use std::{collections::HashMap, path::Path, str::FromStr};
use thiserror::Error;

mod contract;

#[cfg(not(any(test, feature = "nomock")))]
const CONFIG_PATH: &str = "providers.json";
#[cfg(any(test, feature = "nomock"))]
const CONFIG_PATH: &str = "../providers.json";

#[derive(Clone, Deserialize)]
struct Provider {
    pub rpc_url: String,
    pub contract: Address,
}

#[derive(Error, Debug)]
pub enum RpcConfigError {
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),
    #[error("Chain `{0}` is not supported")]
    ChainNotSupported(String),
    #[error("Field `{0}` has not been set")]
    FieldNotSet(String),
}

trait GetProvider {
    fn provider(&self) -> Result<Provider, RpcConfigError>;
}

impl GetProvider for EvmChain {
    fn provider(&self) -> Result<Provider, RpcConfigError> {
        use RpcConfigError::*;

        let settings = Config::builder()
            .add_source(File::from(Path::new(CONFIG_PATH)))
            .build()?;

        let map = settings.try_deserialize::<HashMap<String, Provider>>()?;

        let get_value = |name: &str| {
            let Some(value) = map.get(name) else {
                return Err(FieldNotSet(name.to_string()));
            };

            Ok(value.clone())
        };

        match self {
            EvmChain::Ethereum => get_value("ethereum"),
            EvmChain::Polygon => get_value("polygon"),
            EvmChain::Bsc => get_value("bsc"),
            EvmChain::Gnosis => get_value("gnosis"),
            EvmChain::Arbitrum => get_value("arbitrum"),
            EvmChain::Goerli => get_value("goerli"),
            _ => Err(ChainNotSupported(format!("{self:?}"))),
        }
    }
}

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
    Config(#[from] reqwest::Error),
    #[error(transparent)]
    Reqwest(#[from] RpcConfigError),
    #[error("{0}")]
    Other(String),
}

fn create_payload(method: &str, params: String, id: u32) -> String {
    format!(
        "{{
            \"method\"  : \"{method}\",
            \"params\"  : {params},
            \"id\"      : {id},
            \"jsonrpc\" : \"2.0\"
        }}"
    )
}

async fn get_coin_balance(
    client: &reqwest::Client,
    chain: EvmChain,
    address: Address,
) -> Result<U256, RpcError> {
    let provider = chain
        .provider()
        .map_err(|err| RpcError::Other(err.to_string()))?;

    let payload = create_payload(
        "eth_getBalance",
        format!("[\"{address:?}\", \"latest\"]"),
        1,
    );

    let res: RpcResponse = client
        .post(&provider.rpc_url)
        .body(payload)
        .send()
        .await?
        .json()
        .await?;

    U256::from_str(&res.result).map_err(|err| RpcError::Other(err.to_string()))
}

#[async_trait]
impl BalanceQuerier for RpcProvider {
    type Error = RpcError;
    type Chain = EvmChain;
    type Address = Address;
    type Id = U256;
    type Balance = U256;

    async fn get_balance(
        &self,
        client: &reqwest::Client,
        chain: Self::Chain,
        token_type: TokenType<Self::Address, Self::Id>,
        user_address: Self::Address,
    ) -> Result<Self::Balance, Self::Error> {
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
                None => BalancyProvider
                    .get_balance(client, chain, token_type, user_address)
                    .await
                    .map_err(|err| RpcError::Other(err.to_string())),
            },
        }
    }

    async fn get_balance_batch(
        &self,
        client: &reqwest::Client,
        chain: Self::Chain,
        token_type: TokenType<Self::Address, Self::Id>,
        addresses: &[Self::Address],
    ) -> Result<Vec<Self::Balance>, Self::Error> {
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
                        BalancyProvider
                            .get_balance(client, chain, token_type, *addr)
                            .await
                            .unwrap_or_default()
                    }))
                    .await;

                    Ok(res)
                }
            },
        }
    }
}

#[cfg(all(test, feature = "nomock"))]
mod test {
    use crate::{
        evm::{common::*, jsonrpc::RpcProvider, EvmChain},
        BalanceQuerier,
    };
    use ethereum_types::U256;
    use rusty_gate_common::{address, TokenType::*};

    #[tokio::test]
    async fn rpc_get_coin_balance() {
        assert_eq!(
            RpcProvider
                .get_balance(
                    &reqwest::Client::new(),
                    EvmChain::Ethereum,
                    Native,
                    address!(USER_1_ADDR)
                )
                .await
                .unwrap(),
            U256::from(464468855704627_u128)
        );
    }

    #[tokio::test]
    async fn rpc_get_coin_balance_batch() {
        assert_eq!(
            RpcProvider
                .get_balance_batch(
                    &reqwest::Client::new(),
                    EvmChain::Ethereum,
                    Native,
                    &vec![address!(USER_1_ADDR), address!(USER_2_ADDR)]
                )
                .await
                .unwrap(),
            vec![
                U256::from(464468855704627_u128),
                U256::from(391945502449693859_u128)
            ]
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
                    EvmChain::Ethereum,
                    token_type,
                    address!(USER_2_ADDR)
                )
                .await
                .unwrap(),
            U256::from(100000000000000000000_u128)
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
                    EvmChain::Ethereum,
                    token_type,
                    &vec![address!(USER_1_ADDR), address!(USER_2_ADDR)]
                )
                .await
                .unwrap(),
            vec![U256::from(0), U256::from(100000000000000000000_u128)]
        );
    }

    #[tokio::test]
    async fn rpc_get_erc721_balance() {
        let client = reqwest::Client::new();
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
                .get_balance(
                    &client,
                    EvmChain::Ethereum,
                    token_type_without_id,
                    user_address
                )
                .await
                .unwrap(),
            U256::from(1)
        );
        assert_eq!(
            RpcProvider
                .get_balance(
                    &client,
                    EvmChain::Ethereum,
                    token_type_with_id,
                    user_address
                )
                .await
                .unwrap(),
            U256::from(1)
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
                    EvmChain::Ethereum,
                    token_type_without_id,
                    &vec![address!(USER_1_ADDR), address!(USER_2_ADDR)]
                )
                .await
                .unwrap(),
            vec![U256::from(1), U256::from(1)]
        );
    }

    #[tokio::test]
    async fn rpc_get_erc1155_balance() {
        let client = reqwest::Client::new();
        let token_type_without_id = Special {
            address: address!(ERC1155_ADDR),
            id: None,
        };
        let token_type_with_id = Special {
            address: address!(ERC1155_ADDR),
            id: Some(U256::from(ERC1155_ID)),
        };
        let user_address = address!(USER_3_ADDR);

        assert_eq!(
            RpcProvider
                .get_balance(
                    &client,
                    EvmChain::Ethereum,
                    token_type_without_id,
                    user_address
                )
                .await
                .unwrap(),
            U256::from(6810)
        );
        assert_eq!(
            RpcProvider
                .get_balance(
                    &client,
                    EvmChain::Ethereum,
                    token_type_with_id,
                    user_address
                )
                .await
                .unwrap(),
            U256::from(16)
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
                    EvmChain::Ethereum,
                    token_type_with_id,
                    &vec![address!(USER_1_ADDR), address!(USER_3_ADDR)]
                )
                .await
                .unwrap(),
            vec![U256::from(0), U256::from(16)]
        );
    }
}
