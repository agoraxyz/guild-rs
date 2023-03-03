use crate::{
    evm::{balancy::types::*, get_provider, jsonrpc::get_erc20_decimals},
    BalanceQuerier,
};
use async_trait::async_trait;
use guild_common::{Scalar, TokenType};
use primitive_types::{H160 as Address, U256};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
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
    token: &str,
    address: Address,
) -> Result<BalancyResponse<T>, BalancyError> {
    let Some(id) = get_balancy_id(chain) else {
        return Err(BalancyError::ChainNotSupported(format!("{chain:?}")));
    };

    let res = client
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
    client: &reqwest::Client,
    chain: &str,
    token_address: Address,
    user_address: Address,
) -> Result<Scalar, BalancyError> {
    let tokens = make_balancy_request::<Erc20>(client, chain, "erc20", user_address).await?;
    let decimals = get_erc20_decimals(client, chain, token_address).await?;
    let divider = 10_u128.pow(decimals.as_u32()) as Scalar;

    let amount = tokens
        .result
        .iter()
        .find(|token| token.token_address == token_address)
        .map(|token| (token.amount.as_u128() as Scalar) / divider)
        .unwrap_or_default();

    Ok(amount)
}

async fn get_erc721_balance(
    client: &reqwest::Client,
    chain: &str,
    token_address: Address,
    token_id: Option<U256>,
    user_address: Address,
) -> Result<Scalar, BalancyError> {
    let tokens = make_balancy_request::<Erc721>(client, chain, "erc721", user_address).await?;

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

    Ok(amount as Scalar)
}

async fn get_erc1155_balance(
    client: &reqwest::Client,
    chain: &str,
    token_address: Address,
    token_id: Option<U256>,
    user_address: Address,
) -> Result<Scalar, BalancyError> {
    let tokens = make_balancy_request::<Erc1155>(client, chain, "erc1155", user_address).await?;

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
        .map(|token| token.amount.as_u128())
        .reduce(|a, b| a + b)
        .unwrap_or_default();

    Ok(amount as Scalar)
}

#[async_trait]
impl BalanceQuerier for BalancyProvider {
    type Error = BalancyError;
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
            TokenType::Fungible { address } => {
                get_erc20_balance(client, chain, address, user_address).await
            }
            TokenType::NonFungible { address, id } => {
                get_erc721_balance(client, chain, address, id, user_address).await
            }
            TokenType::Special { address, id } => {
                get_erc1155_balance(client, chain, address, id, user_address).await
            }
            TokenType::Native => Err(BalancyError::TokenTypeNotSupported(format!(
                "{token_type:?}"
            ))),
        }
    }

    async fn get_balance_batch(
        &self,
        client: &reqwest::Client,
        chain: &str,
        token_type: TokenType<Self::Address, Self::Id>,
        addresses: &[Self::Address],
    ) -> Result<Vec<Scalar>, Self::Error> {
        Ok(
            futures::future::join_all(addresses.iter().map(|address| async {
                self.get_balance(client, chain, token_type, *address)
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
        address,
        Chain::{Bsc, Ethereum, Gnosis, Goerli, Polygon},
        TokenType::*,
    };
    use primitive_types::U256;

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
            "erc20",
            address!(USER_1_ADDR)
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn balancy_bsc() {
        assert!(make_balancy_request::<Erc20>(
            &reqwest::Client::new(),
            &Bsc.to_string(),
            "erc20",
            address!(USER_1_ADDR)
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn balancy_gnosis() {
        assert!(make_balancy_request::<Erc20>(
            &reqwest::Client::new(),
            &Gnosis.to_string(),
            "erc20",
            address!(USER_1_ADDR)
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn balancy_polygon() {
        assert!(make_balancy_request::<Erc20>(
            &reqwest::Client::new(),
            &Polygon.to_string(),
            "erc20",
            address!(USER_1_ADDR)
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn balancy_get_erc20_balance() {
        let token_type = Fungible {
            address: address!(ERC20_ADDR),
        };

        assert_eq!(
            BalancyProvider
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
    async fn balancy_get_erc721_balance() {
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
            BalancyProvider
                .get_balance(&client, &chain, token_type_without_id, user_address)
                .await
                .unwrap(),
            1.0
        );
        assert_eq!(
            BalancyProvider
                .get_balance(&client, &chain, token_type_with_id, user_address)
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
            address: address!(ERC1155_ADDR),
            id: None,
        };
        let token_type_with_id = Special {
            address: address!(ERC1155_ADDR),
            id: Some(U256::from(ERC1155_ID)),
        };
        let user_address = address!(USER_3_ADDR);

        assert!(
            BalancyProvider
                .get_balance(&client, &chain, token_type_without_id, user_address)
                .await
                .unwrap()
                > 6000.0
        );
        assert_eq!(
            BalancyProvider
                .get_balance(&client, &chain, token_type_with_id, user_address)
                .await
                .unwrap(),
            16.0
        );
    }
}
