use crate::{
    evm::{balancy::types::*, EvmChain},
    BalanceQuerier,
};
use async_trait::async_trait;
use ethereum_types::{Address, U256};
use reqwest::StatusCode;
use rusty_gate_common::TokenType;
use std::collections::HashMap;
use tokio::sync::RwLock;

mod types;

const BASE_URL: &str = "https://balancy.guild.xyz/api";
const ADDRESS_TOKENS: &str = "addressTokens?address=";
const BALANCY_CHAIN: &str = "&chain=";

lazy_static::lazy_static! {
    static ref CLIENT: RwLock<reqwest::Client> =
        RwLock::new(reqwest::Client::new());
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

pub async fn get_address_tokens(
    chain: EvmChain,
    address: Address,
) -> Result<AddressTokenResponse, BalancyError> {
    match CHAIN_IDS.get(&(chain as u32)) {
        None => Err(BalancyError::ChainNotSupported(format!("{chain:?}"))),
        Some(id) => {
            let res = CLIENT
                .read()
                .await
                .get(format!(
                    "{BASE_URL}/{ADDRESS_TOKENS}{address:#x}{BALANCY_CHAIN}{id}"
                ))
                .send()
                .await?;

            let status = res.status();

            match status {
                StatusCode::OK => Ok(res.json::<AddressTokenResponse>().await?),
                StatusCode::BAD_REQUEST => Err(BalancyError::InvalidBalancyRequest),
                StatusCode::TOO_MANY_REQUESTS => Err(BalancyError::TooManyRequests),
                _ => Err(BalancyError::Unknown(status.as_u16())),
            }
        }
    }
}

#[async_trait]
impl BalanceQuerier for BalancyProvider {
    type Error = BalancyError;
    type Address = ethereum_types::H160;

    async fn get_balance_for_many(
        &self,
        token_type: TokenType,
        addresses: &[Self::Address],
    ) -> Result<Vec<U256>, Self::Error> {
        Ok(
            futures::future::join_all(addresses.iter().map(|address| async {
                self.get_balance_for_one(token_type, *address)
                    .await
                    .unwrap_or(U256::from(0))
            }))
            .await,
        )
    }

    async fn get_balance_for_one(
        &self,
        token_type: TokenType,
        address: Self::Address,
    ) -> Result<U256, Self::Error> {
        let tokens = get_address_tokens(EvmChain::Ethereum, address).await?;

        match token_type {
            TokenType::Fungible { address } => {
                let res = tokens
                    .erc20
                    .iter()
                    .find(|token| token.address == address)
                    .map(|token| token.amount)
                    .unwrap_or_default();

                Ok(res)
            }
            TokenType::NonFungible { address, id } => {
                let res = tokens
                    .erc721
                    .iter()
                    .filter(|token| {
                        token.address == address && {
                            match id {
                                Some(id) => token.token_id == id,
                                None => true,
                            }
                        }
                    })
                    .count();

                Ok(U256::from(res))
            }
            TokenType::Special { address, id } => {
                let res = tokens
                    .erc1155
                    .iter()
                    .filter(|token| {
                        token.addr == address && {
                            match id {
                                Some(id) => token.token_id == id,
                                None => true,
                            }
                        }
                    })
                    .map(|token| token.amount)
                    .reduce(|a, b| a + b)
                    .unwrap_or(U256::from(0));

                Ok(res)
            }
            TokenType::Coin => Err(BalancyError::TokenTypeNotSupported("COIN".to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::{evm::balancy::get_address_tokens, evm::EvmChain};
    use ethereum_types::Address;

    #[tokio::test]
    async fn balancy_address_tokens() {
        assert!(get_address_tokens(
            EvmChain::Ethereum,
            Address::from_str("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE").unwrap()
        )
        .await
        .is_ok());
    }
}
