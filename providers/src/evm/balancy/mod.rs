use crate::{
    evm::{balancy::types::*, get_provider, jsonrpc::get_erc20_decimals},
    BalanceQuerier,
};
use async_trait::async_trait;
use guild_common::{Scalar, TokenType};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use std::any::TypeId;
pub use types::BalancyError;

mod types;

const BASE_URL: &str = "https://balancy.guild.xyz/api";
const ADDRESS_TOKENS: &str = "addressTokens?address=";
const BALANCY_CHAIN: &str = "&chain=";

pub struct BalancyProvider;

fn get_balancy_id(chain: &str) -> Option<u8> {
    match get_provider(chain) {
        Ok(provider) => provider.balancy_id,
        Err(_) => None,
    }
}

async fn make_balancy_request<T: DeserializeOwned + 'static>(
    client: &reqwest::Client,
    chain: &str,
    address: &str,
) -> Result<BalancyResponse<T>, BalancyError> {
    let Some(id) = get_balancy_id(chain) else {
        return Err(BalancyError::ChainNotSupported(chain.to_string()));
    };

    let token = if TypeId::of::<T>() == TypeId::of::<Erc20>() {
        "erc20"
    } else if TypeId::of::<T>() == TypeId::of::<Erc721>() {
        "erc721"
    } else if TypeId::of::<T>() == TypeId::of::<Erc1155>() {
        "erc1155"
    } else {
        "coin"
    };

    let res = client
        .get(format!(
            "{BASE_URL}/{token}/{ADDRESS_TOKENS}{address}{BALANCY_CHAIN}{id}"
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
    client: &reqwest::Client,
    chain: &str,
    token_address: &str,
    user_address: &str,
) -> Result<Scalar, BalancyError> {
    let tokens = make_balancy_request::<Erc20>(client, chain, user_address).await?;
    let decimals = get_erc20_decimals(client, chain, token_address).await?;
    let divider = 10_u128.pow(decimals) as Scalar;

    let amount = tokens
        .result
        .iter()
        .find(|token| token.token_address.to_lowercase() == token_address.to_lowercase())
        .map(|token| (token.amount.as_u128() as Scalar) / divider)
        .unwrap_or_default();

    Ok(amount)
}

async fn get_erc721_balance(
    client: &reqwest::Client,
    chain: &str,
    token_address: &str,
    token_id: Option<String>,
    user_address: &str,
) -> Result<Scalar, BalancyError> {
    let tokens = make_balancy_request::<Erc721>(client, chain, user_address).await?;

    let amount = tokens
        .result
        .iter()
        .filter(|token| {
            token.token_address.to_lowercase() == token_address.to_lowercase() && {
                match token_id.as_ref() {
                    Some(id) => &token.token_id == id,
                    None => true,
                }
            }
        })
        .count();

    Ok(amount as Scalar)
}

async fn get_erc1155_balance(
    client: &reqwest::Client,
    chain: &str,
    token_address: &str,
    token_id: Option<String>,
    user_address: &str,
) -> Result<Scalar, BalancyError> {
    let tokens = make_balancy_request::<Erc1155>(client, chain, user_address).await?;

    let amount = tokens
        .result
        .iter()
        .filter(|token| {
            token.token_address.to_lowercase() == token_address.to_lowercase() && {
                match token_id.as_ref() {
                    Some(id) => &token.token_id == id,
                    None => true,
                }
            }
        })
        .map(|token| token.amount.as_u128())
        .reduce(|a, b| a + b)
        .unwrap_or_default();

    Ok(amount as Scalar)
}

#[async_trait]
impl BalanceQuerier for BalancyProvider {
    type Error = BalancyError;

    async fn get_balance(
        &self,
        client: &reqwest::Client,
        chain: &str,
        token_type: &TokenType,
        user_address: &str,
    ) -> Result<Scalar, Self::Error> {
        match token_type {
            TokenType::Fungible { address } => {
                get_erc20_balance(client, chain, &address, user_address).await
            }
            TokenType::NonFungible { address, id } => {
                get_erc721_balance(client, chain, &address, id.clone(), user_address).await
            }
            TokenType::Special { address, id } => {
                get_erc1155_balance(client, chain, &address, id.clone(), user_address).await
            }
            TokenType::Native => Err(BalancyError::NativeTokenNotSupported),
        }
    }

    async fn get_balance_batch(
        &self,
        client: &reqwest::Client,
        chain: &str,
        token_type: &TokenType,
        addresses: &[String],
    ) -> Result<Vec<Scalar>, Self::Error> {
        Ok(
            futures::future::join_all(addresses.iter().map(|address| async {
                self.get_balance(client, chain, token_type, address)
                    .await
                    .unwrap_or_default()
            }))
            .await,
        )
    }
}

#[cfg(all(test, feature = "nomock"))]
mod test {
    use crate::{
        evm::{
            balancy::{get_balancy_id, make_balancy_request, types::Erc20, BalancyProvider},
            common::*,
        },
        BalanceQuerier,
    };
    use guild_common::{
        Chain::{Bsc, Ethereum, Gnosis, Goerli, Polygon},
        TokenType::*,
    };

    #[test]
    fn balancy_get_chain_id() {
        assert_eq!(get_balancy_id(&Ethereum.to_string()), Some(1));
        assert_eq!(get_balancy_id(&Bsc.to_string()), Some(56));
        assert_eq!(get_balancy_id(&Gnosis.to_string()), Some(100));
        assert_eq!(get_balancy_id(&Polygon.to_string()), Some(137));
        assert_eq!(get_balancy_id(&Goerli.to_string()), None);
    }

    #[tokio::test]
    async fn balancy_ethereum() {
        assert!(make_balancy_request::<Erc20>(
            &reqwest::Client::new(),
            &Ethereum.to_string(),
            USER_1_ADDR
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn balancy_bsc() {
        assert!(make_balancy_request::<Erc20>(
            &reqwest::Client::new(),
            &Bsc.to_string(),
            USER_1_ADDR
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn balancy_gnosis() {
        assert!(make_balancy_request::<Erc20>(
            &reqwest::Client::new(),
            &Gnosis.to_string(),
            USER_1_ADDR
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn balancy_polygon() {
        assert!(make_balancy_request::<Erc20>(
            &reqwest::Client::new(),
            &Polygon.to_string(),
            USER_1_ADDR
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn balancy_get_erc20_balance() {
        let token_type = Fungible {
            address: ERC20_ADDR.to_string(),
        };

        assert_eq!(
            BalancyProvider
                .get_balance(
                    &reqwest::Client::new(),
                    &Ethereum.to_string(),
                    &token_type,
                    USER_2_ADDR
                )
                .await
                .unwrap(),
            100.0
        );
    }

    #[tokio::test]
    async fn balancy_get_erc721_balance() {
        let client = reqwest::Client::new();
        let chain = Ethereum.to_string();
        let token_type_without_id = NonFungible {
            address: ERC721_ADDR.to_string(),
            id: None,
        };
        let token_type_with_id = NonFungible {
            address: ERC721_ADDR.to_string(),
            id: Some(ERC721_ID.to_string()),
        };

        assert_eq!(
            BalancyProvider
                .get_balance(&client, &chain, &token_type_without_id, USER_1_ADDR)
                .await
                .unwrap(),
            1.0
        );
        assert_eq!(
            BalancyProvider
                .get_balance(&client, &chain, &token_type_with_id, USER_1_ADDR)
                .await
                .unwrap(),
            1.0
        );
    }

    #[tokio::test]
    async fn balancy_get_erc1155_balance() {
        let client = reqwest::Client::new();
        let chain = Ethereum.to_string();
        let token_type_without_id = Special {
            address: ERC1155_ADDR.to_string(),
            id: None,
        };
        let token_type_with_id = Special {
            address: ERC1155_ADDR.to_string(),
            id: Some(ERC1155_ID.to_string()),
        };

        assert!(
            BalancyProvider
                .get_balance(&client, &chain, &token_type_without_id, USER_3_ADDR)
                .await
                .unwrap()
                > 6000.0
        );
        assert_eq!(
            BalancyProvider
                .get_balance(&client, &chain, &token_type_with_id, USER_3_ADDR)
                .await
                .unwrap(),
            16.0
        );
    }
}
