use crate::balances::Balances;
use crate::call::*;
use guild_requirement::token::TokenType;
use reqwest::Client;
use serde::Deserialize;
use zeroize::Zeroize;

#[derive(Clone, Debug, Deserialize)]
pub struct Provider {
    pub rpc_url: String,
    pub target: String,
}

impl Provider {
    pub async fn balances(
        self,
        client: Client,
        token_type: TokenType,
        addresses: &[String],
    ) -> Result<Balances, anyhow::Error> {
        let Provider {
            mut rpc_url,
            target,
        } = self;
        let balances = match token_type {
            TokenType::Native => eth_balances(client, addresses, target, &rpc_url).await,
            TokenType::Fungible { address } => {
                erc20_balances(client, addresses, target, address, &rpc_url).await
            }
            TokenType::NonFungible {
                address,
                id: maybe_id,
            } => match maybe_id {
                Some(id) => erc721_ownership(client, addresses, address, id, &rpc_url).await,
                None => erc721_balances(client, addresses, target, address, &rpc_url).await,
            },
            TokenType::Special {
                address,
                id: maybe_id,
            } => match maybe_id {
                Some(id) => erc1155_balances(client, addresses, address, id, &rpc_url).await,
                None => Ok(Balances::new(vec![0.0; addresses.len()])),
            },
        };
        rpc_url.zeroize();
        balances
    }
}
