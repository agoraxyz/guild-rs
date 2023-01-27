use crate::{
    evm::{balancy::types::*, EvmChain},
    BalanceQuerier,
};
use async_trait::async_trait;
use ethereum_types::{Address, U256};
use reqwest::StatusCode;
use rusty_gate_common::TokenType;
use serde::de::DeserializeOwned;
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

async fn make_balancy_request<T: DeserializeOwned + 'static>(
    chain: EvmChain,
    token: &str,
    address: Address,
) -> Result<BalancyResponse<T>, BalancyError> {
    let Some(id) = CHAIN_IDS.get(&(chain as u32)) else {
        return Err(BalancyError::ChainNotSupported(format!("{chain:?}")));
    };

    let res = CLIENT
        .read()
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
                let tokens = make_balancy_request::<Erc20>(chain, "erc20", user_address).await?;

                let amount = tokens
                    .result
                    .iter()
                    .find(|token| token.token_address == address)
                    .map(|token| token.amount)
                    .unwrap_or_default();

                Ok(amount)
            }
            TokenType::NonFungible { address, id } => {
                let tokens = make_balancy_request::<Erc721>(chain, "erc721", user_address).await?;

                let amount = tokens
                    .result
                    .iter()
                    .filter(|token| {
                        token.token_address == address && {
                            match id {
                                Some(id) => token.token_id == id,
                                None => true,
                            }
                        }
                    })
                    .count();

                Ok(U256::from(amount))
            }
            TokenType::Special { address, id } => {
                let tokens =
                    make_balancy_request::<Erc1155>(chain, "erc1155", user_address).await?;

                let amount = tokens
                    .result
                    .iter()
                    .filter(|token| {
                        token.token_address == address && {
                            match id {
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
            EvmChain,
        },
        BalanceQuerier,
    };
    use ethereum_types::U256;
    use rusty_gate_common::{address, TokenType::*};

    const USER_1_ADDR: &str = "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE";
    const USER_2_ADDR: &str = "0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3";
    const USER_3_ADDR: &str = "0x283d678711daa088640c86a1ad3f12c00ec1252e";
    const ERC20_ADDR: &str = "0x458691c1692cd82facfb2c5127e36d63213448a8";
    const ERC721_ADDR: &str = "0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85";
    const ERC721_ID: &str =
        "61313325075603536901663283754390960556726744542208800735045237225934362163454";
    const ERC1155_ADDR: &str = "0x76be3b62873462d2142405439777e971754e8e77";
    const ERC1155_ID: &str = "10868";

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
            U256::from(6840)
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
