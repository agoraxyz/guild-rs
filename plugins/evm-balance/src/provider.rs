use crate::balances::Balances;
use guild_common::Scalar;
use guild_requirement::token::TokenType;
use reqwest::Client;
use serde::Deserialize;
use zeroize::ZeroizeOnDrop;

#[derive(Clone, Debug, Deserialize, ZeroizeOnDrop)]
pub struct Provider {
    pub rpc_url: String,
    #[zeroize(skip)]
    pub contract: String,
}

impl Provider {
    pub async fn get_balance_batch(
        &self,
        client: Client,
        token_type: TokenType,
        addresses: &[String],
    ) -> Result<Balances, anyhow::Error> {
        match token_type {
            _ => todo!()
            //TokenType::Native => {
            //    get_eth_balance_batch(client, &self.contract.to_string(), &self.rpc_url, addresses)
            //        .await
            //}
            //TokenType::Fungible { address } => {
            //    get_erc20_balance_batch(
            //        client,
            //        &self.contract.to_string(),
            //        &self.rpc_url,
            //        &address,
            //        addresses,
            //    )
            //    .await
            //}
            //TokenType::NonFungible { address, id: _ } => {
            //    get_erc721_balance_batch(
            //        client,
            //        &self.contract.to_string(),
            //        &self.rpc_url,
            //        &address,
            //        addresses,
            //    )
            //    .await
            //}
            //TokenType::Special { address, id } => match id {
            //    Some(token_id) => {
            //        get_erc1155_balance_batch(
            //            client,
            //            &self.rpc_url,
            //            address.clone(),
            //            &token_id,
            //            addresses,
            //        )
            //        .await
            //    }
            //    None => Ok(vec![0.0; addresses.len()]),
            //},
        }
    }
}
