use crate::{
    evm::{balancy::BALANCY_PROVIDER, jsonrpc::contract::*, EvmChain},
    BalanceQuerier, TokenType, CLIENT,
};
use async_trait::async_trait;
use ethereum_types::{Address, U256};
use rusty_gate_common::address;
use serde::Deserialize;
use std::str::FromStr;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

mod contract;

struct Provider {
    pub rpc_url: String,
    // TODO: implement multicall
    // https://github.com/agoraxyz/requirement-engine-v2/issues/9
    #[allow(dead_code)]
    pub multicall_contract: Address,
}

macro_rules! dotenv {
    ($var: expr) => {
        match std::env::var($var) {
            Ok(val) => val,
            Err(_) => panic!("Environment variable `{}` not found", $var),
        }
    };
}

lazy_static::lazy_static! {
    static ref PROVIDERS: Arc<HashMap<EvmChain, Provider>> = Arc::new({
        dotenv::dotenv().ok();

        let mut providers = HashMap::new();

        providers.insert(
            EvmChain::Ethereum,
            Provider {
                rpc_url: dotenv!("ETHEREUM_RPC"),
                multicall_contract: address!("0x5ba1e12693dc8f9c48aad8770482f4739beed696"),
            }
        );
        providers.insert(
            EvmChain::Polygon,
            Provider {
                rpc_url: dotenv!("POLYGON_RPC"),
                multicall_contract: address!("0x11ce4B23bD875D7F5C6a31084f55fDe1e9A87507"),
            }
        );
        providers.insert(
            EvmChain::Bsc,
            Provider {
                rpc_url: dotenv!("BSC_RPC"),
                multicall_contract: address!("0x41263cba59eb80dc200f3e2544eda4ed6a90e76c")
            }
        );
        providers.insert(
            EvmChain::Gnosis,
            Provider {
                rpc_url: dotenv!("GNOSIS_RPC"),
                multicall_contract: address!("0xb5b692a88bdfc81ca69dcb1d924f59f0413a602a")
            }
        );
        providers.insert(
            EvmChain::Arbitrum,
            Provider {
                rpc_url: dotenv!("ARBITRUM_RPC"),
                multicall_contract: address!("0x52bfe8fE06c8197a8e3dCcE57cE012e13a7315EB")
            }
        );
        providers.insert(
            EvmChain::Goerli,
            Provider {
                rpc_url: dotenv!("GOERLI_RPC"),
                multicall_contract: address!("0x77dCa2C955b15e9dE4dbBCf1246B4B85b651e50e")
            }
        );

        providers
    });
}

#[derive(Deserialize)]
pub struct RpcResponse {
    pub result: String,
}

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

async fn get_coin_balance(chain: EvmChain, address: Address) -> Result<U256, RpcError> {
    let Some(provider) = PROVIDERS.get(&chain) else {
       return Err(RpcError::ChainNotSupported(format!("{chain:?}")));
    };

    let payload = create_payload(
        "eth_getBalance",
        format!("[\"{address:?}\", \"latest\"]"),
        1,
    );

    let res: RpcResponse = CLIENT
        .read()
        .await
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
            TokenType::Special { address, id } => match id {
                Some(token_id) => get_erc1155_balance(chain, address, token_id, user_address).await,
                None => BALANCY_PROVIDER
                    .get_balance_for_one(chain, token_type, user_address)
                    .await
                    .map_err(|err| RpcError::Other(err.to_string())),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        evm::{common::*, jsonrpc::RPC_PROVIDER, EvmChain},
        BalanceQuerier,
    };
    use ethereum_types::U256;
    use rusty_gate_common::{address, TokenType::*};

    #[tokio::test]
    async fn rpc_get_coin_balance() {
        assert!(RPC_PROVIDER
            .get_balance_for_one(
                EvmChain::Ethereum,
                Coin,
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
            RPC_PROVIDER
                .get_balance_for_one(EvmChain::Ethereum, token_type, address!(USER_2_ADDR))
                .await
                .unwrap(),
            U256::from(100000000000000000000_u128)
        );
    }

    #[tokio::test]
    async fn rpc_get_erc721_balance() {
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
            RPC_PROVIDER
                .get_balance_for_one(EvmChain::Ethereum, token_type_without_id, user_address)
                .await
                .unwrap(),
            U256::from(1)
        );
        assert_eq!(
            RPC_PROVIDER
                .get_balance_for_one(EvmChain::Ethereum, token_type_with_id, user_address)
                .await
                .unwrap(),
            U256::from(1)
        );
    }

    #[tokio::test]
    async fn rpc_get_erc1155_balance() {
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
            RPC_PROVIDER
                .get_balance_for_one(EvmChain::Ethereum, token_type_without_id, user_address)
                .await
                .unwrap(),
            U256::from(6830)
        );
        assert_eq!(
            RPC_PROVIDER
                .get_balance_for_one(EvmChain::Ethereum, token_type_with_id, user_address)
                .await
                .unwrap(),
            U256::from(16)
        );
    }
}
