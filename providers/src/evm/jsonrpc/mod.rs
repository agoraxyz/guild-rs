use crate::{
    evm::{balancy::BalancyProvider, jsonrpc::contract::*, EvmChain},
    BalanceQuerier, TokenType,
};
use async_trait::async_trait;
use config::{Config, File};
pub use contract::get_erc20_decimals;
use futures::future::join_all;
use guild_common::Scalar;
use primitive_types::{H160 as Address, U256};
use serde::Deserialize;
use std::{collections::HashMap, path::Path, str::FromStr};
use thiserror::Error;

mod contract;

#[cfg(not(any(test, feature = "nomock")))]
const CONFIG_PATH: &str = "providers.json";
#[cfg(any(test, feature = "nomock"))]
const CONFIG_PATH: &str = "../providers.json";
const ETH_BALANCE_DIVIDER: Scalar = 10_u128.pow(18) as Scalar;

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
) -> Result<Scalar, RpcError> {
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

    let balance = U256::from_str(&res.result).map_err(|err| RpcError::Other(err.to_string()))?;

    Ok((balance.as_u128() as f64) / ETH_BALANCE_DIVIDER)
}

#[async_trait]
impl BalanceQuerier for RpcProvider {
    type Error = RpcError;
    type Chain = EvmChain;
    type Address = Address;
    type Id = U256;

    async fn get_balance(
        &self,
        client: &reqwest::Client,
        chain: Self::Chain,
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
    use guild_common::{address, TokenType::*};
    use primitive_types::U256;

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
            0.000464468855704627
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
                    EvmChain::Ethereum,
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
                    EvmChain::Ethereum,
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
            1.0
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
                    EvmChain::Ethereum,
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
            6730.0
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
                    EvmChain::Ethereum,
                    token_type_with_id,
                    &vec![address!(USER_1_ADDR), address!(USER_3_ADDR)]
                )
                .await
                .unwrap(),
            vec![0.0, 16.0]
        );
    }
}
