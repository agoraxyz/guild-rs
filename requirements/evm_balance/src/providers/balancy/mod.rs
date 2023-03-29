use crate::{providers::balancy::types::*};
use reqwest::StatusCode;
pub use types::BalancyError;
use reqwest::Client;

mod types;

const BASE_URL: &str = "https://balancy.guild.xyz/api";
const ADDRESS_TOKENS: &str = "addressTokens?address=";
const BALANCY_CHAIN: &str = "&chain=";

async fn make_balancy_request(
    client: &'static Client,
    chain_id: u8,
    address: &str,
) -> Result<BalancyResponse, BalancyError> {
    let id = chain_id;

    let res = client
        .get(format!(
            "{BASE_URL}/erc1155/{ADDRESS_TOKENS}{address}{BALANCY_CHAIN}{id}"
        ))
        .send()
        .await?;

    let status = res.status();

    match status {
        StatusCode::OK => Ok(res.json::<BalancyResponse>().await?),
        StatusCode::BAD_REQUEST => Err(BalancyError::InvalidBalancyRequest),
        StatusCode::TOO_MANY_REQUESTS => Err(BalancyError::TooManyRequests),
        _ => Err(BalancyError::Unknown(status.as_u16())),
    }
}

pub async fn get_erc1155_balance(
    client: &'static Client,
    chain_id: u8,
    token_address: &str,
    user_address: &str,
) -> Result<f64, BalancyError> {
    let tokens = make_balancy_request(client, chain_id, user_address).await?;

    let amount = tokens
        .result
        .iter()
        .filter(|token| token.token_address.to_lowercase() == token_address.to_lowercase())
        .map(|token| token.amount.as_u128())
        .reduce(|a, b| a + b)
        .unwrap_or_default();

    Ok(amount as f64)
}

#[cfg(all(test, feature = "nomock"))]
mod test {
    use crate::providers::{
        balancy::get_erc1155_balance,
        common::{ERC1155_ADDR, USER_3_ADDR},
    };
    use reqwest::Client;

    #[tokio::test]
    async fn balancy_get_erc1155_balance() {
        let client: &'static Client = Box::leak(Box::new(Client::new()));

        assert!(
            get_erc1155_balance(client, 1, ERC1155_ADDR, USER_3_ADDR)
                .await
                .unwrap()
                > 6000.0
        );
    }
}
