use crate::{
    evm::{balancy::types::*, EvmChain},
    BalanceQuerier, CLIENT,
};
use async_trait::async_trait;
use ethereum_types::{Address, U256};
use reqwest::StatusCode;
use rusty_gate_common::TokenType;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

mod types;

const BASE_URL: &str = "https://balancy.guild.xyz/api";
const ADDRESS_TOKENS: &str = "addressTokens?address=";
const BALANCY_CHAIN: &str = "&chain=";

lazy_static::lazy_static! {
    static ref CHAIN_IDS: HashMap<u32, u32> = {
        let mut h = HashMap::new();

        h.insert(EvmChain::Ethereum as u32, 1);
        h.insert(EvmChain::Bsc as u32, 56);
        h.insert(EvmChain::Gnosis as u32, 100);
        h.insert(EvmChain::Polygon as u32, 137);

        h
    };
}

pub struct BalancyProvider;

pub const BALANCY_PROVIDER: BalancyProvider = BalancyProvider {};

async fn make_balancy_request<T: DeserializeOwned + 'static>(
    chain: EvmChain,
    token: &str,
    address: Address,
) -> Result<BalancyResponse<T>, BalancyError> {
    let Some(id) = CHAIN_IDS.get(&(chain as u32)) else {
        return Err(BalancyError::ChainNotSupported(format!("{chain:?}")));
    };

    let res = CLIENT
        .write()
        .await
        .get(format!(
            "{BASE_URL}/{token}/{ADDRESS_TOKENS}{address:#x}{BALANCY_CHAIN}{id}"
        ))
        .send()
        .await?;

    let status = res.status();

    match status {
        StatusCode::OK => Ok(res.json::<BalancyResponse<T>>().await?),
        StatusCode::BAD_REQUEST => Err(BalancyError::InvalidBalancyRequest),
        StatusCode::TOO_MANY_REQUESTS => Err(BalancyError::TooManyRequests),
        _ => Err(BalancyError::Unknown(status.as_u16())),
    }
}

async fn get_erc20_balance(
    chain: EvmChain,
    token_address: Address,
    user_address: Address,
) -> Result<U256, BalancyError> {
    let tokens = make_balancy_request::<Erc20>(chain, "erc20", user_address).await?;

    let amount = tokens
        .result
        .iter()
        .find(|token| token.token_address == token_address)
        .map(|token| token.amount)
        .unwrap_or_default();

    Ok(amount)
}

async fn get_erc721_balance(
    chain: EvmChain,
    token_address: Address,
    token_id: Option<U256>,
    user_address: Address,
) -> Result<U256, BalancyError> {
    let tokens = make_balancy_request::<Erc721>(chain, "erc721", user_address).await?;

    let amount = tokens
        .result
        .iter()
        .filter(|token| {
            token.token_address == token_address && {
                match token_id {
                    Some(id) => token.token_id == id,
                    None => true,
                }
            }
        })
        .count();

    Ok(U256::from(amount))
}

async fn get_erc1155_balance(
    chain: EvmChain,
    token_address: Address,
    token_id: Option<U256>,
    user_address: Address,
) -> Result<U256, BalancyError> {
    let tokens = make_balancy_request::<Erc1155>(chain, "erc1155", user_address).await?;

    let amount = tokens
        .result
        .iter()
        .filter(|token| {
            token.token_address == token_address && {
                match token_id {
                    Some(id) => token.token_id == id,
                    None => true,
                }
            }
        })
        .map(|token| token.amount)
        .reduce(|a, b| a + b)
        .unwrap_or_default();

    Ok(amount)
}

#[async_trait]
impl BalanceQuerier for BalancyProvider {
    type Error = BalancyError;
    type Address = ethereum_types::H160;

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
            TokenType::Fungible { address } => {
                get_erc20_balance(chain, address, user_address).await
            }
            TokenType::NonFungible { address, id } => {
                get_erc721_balance(chain, address, id, user_address).await
            }
            TokenType::Special { address, id } => {
                get_erc1155_balance(chain, address, id, user_address).await
            }
            TokenType::Coin => Err(BalancyError::TokenTypeNotSupported(format!(
                "{token_type:?}"
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        evm::{
            balancy::{make_balancy_request, types::Erc20, BALANCY_PROVIDER},
            common::*,
            EvmChain,
        },
        BalanceQuerier,
    };
    use ethereum_types::U256;
    use rusty_gate_common::{address, TokenType::*};

    #[tokio::test]
    async fn balancy_ethereum() {
        assert!(
            make_balancy_request::<Erc20>(EvmChain::Ethereum, "erc20", address!(USER_1_ADDR))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn balancy_bsc() {
        assert!(
            make_balancy_request::<Erc20>(EvmChain::Bsc, "erc20", address!(USER_1_ADDR))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn balancy_gnosis() {
        assert!(
            make_balancy_request::<Erc20>(EvmChain::Gnosis, "erc20", address!(USER_1_ADDR))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn balancy_polygon() {
        assert!(
            make_balancy_request::<Erc20>(EvmChain::Polygon, "erc20", address!(USER_1_ADDR))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn balancy_get_erc20_balance() {
        let token_type = Fungible {
            address: address!(ERC20_ADDR),
        };

        assert_eq!(
            BALANCY_PROVIDER
                .get_balance_for_one(EvmChain::Ethereum, token_type, address!(USER_2_ADDR))
                .await
                .unwrap(),
            U256::from(100000000000000000000_u128)
        );
    }

    #[tokio::test]
    async fn balancy_get_erc721_balance() {
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
            BALANCY_PROVIDER
                .get_balance_for_one(EvmChain::Ethereum, token_type_without_id, user_address)
                .await
                .unwrap(),
            U256::from(1)
        );
        assert_eq!(
            BALANCY_PROVIDER
                .get_balance_for_one(EvmChain::Ethereum, token_type_with_id, user_address)
                .await
                .unwrap(),
            U256::from(1)
        );
    }

    #[tokio::test]
    async fn balancy_get_erc1155_balance() {
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
            BALANCY_PROVIDER
                .get_balance_for_one(EvmChain::Ethereum, token_type_without_id, user_address)
                .await
                .unwrap(),
            U256::from(6830)
        );
        assert_eq!(
            BALANCY_PROVIDER
                .get_balance_for_one(EvmChain::Ethereum, token_type_with_id, user_address)
                .await
                .unwrap(),
            U256::from(16)
        );
    }
}
