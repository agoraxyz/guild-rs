use crate::{
    evm::{balancy::BalancyProvider, jsonrpc::contract::*, EvmChain},
    BalanceQuerier, TokenType,
};
use async_trait::async_trait;
use config::{Config, File};
pub use contract::get_erc20_decimals;
use ethereum_types::{Address, U256};
use serde::Deserialize;
use std::{collections::HashMap, path::Path, str::FromStr};
use thiserror::Error;

mod contract;

#[derive(Clone, Deserialize)]
struct Provider {
    pub rpc_url: String,
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

        #[cfg(not(any(test, feature = "nomock")))]
        let path = "providers.json";
        #[cfg(any(test, feature = "nomock"))]
        let path = "../providers.json";

        let settings = Config::builder()
            .add_source(File::from(Path::new(path)))
            .build()?;

        let map = settings.try_deserialize::<HashMap<String, Provider>>()?;

        let get_value = |name: &str| {
            let Some(value) = map.get(name) else {
                return Err(FieldNotSet(name.to_string()));
            };

            Ok(value.clone())
        };

        match self {
            EvmChain::Ethereum => get_value("ETHEREUM_RPC"),
            EvmChain::Polygon => get_value("POLYGON_RPC"),
            EvmChain::Bsc => get_value("BSC_RPC"),
            EvmChain::Gnosis => get_value("GNOSIS_RPC"),
            EvmChain::Arbitrum => get_value("ARBITRUM_RPC"),
            EvmChain::Goerli => get_value("GOERLI_RPC"),
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
    Reqwest(#[from] reqwest::Error),
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
        Ok(
            futures::future::join_all(addresses.iter().map(|address| async {
                self.get_balance(client, chain, token_type, *address)
                    .await
                    .unwrap_or(U256::from(0))
            }))
            .await,
        )
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
        assert!(RpcProvider
            .get_balance(
                &reqwest::Client::new(),
                EvmChain::Ethereum,
                Native,
                address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")
            )
            .await
            .is_ok());
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
    async fn rpc_get_erc1155_balance() {
        let client = reqwest::Client::new();
        let token_type_without_id = Special {
            address: address!(ERC1155_ADDR),
            id: None,
        };
        let token_type_with_id = Special {
            address: address!(ERC1155_ADDR),
            id: Some(U256::from_dec_str(ERC1155_ID).unwrap()),
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
}
