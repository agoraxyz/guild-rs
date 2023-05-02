use crate::balances::Balances;
use crate::call::Multicall;
use guild_requirement::token::TokenType;
use reqwest::Client;
use serde::Deserialize;
use zeroize::Zeroize;

use std::str::FromStr;

#[derive(Clone, Debug, Deserialize)]
pub struct Provider {
    pub rpc_url: String,
    pub contract: String,
}

impl Provider {
    pub async fn get_balance_batch(
        mut self,
        client: Client,
        token_type: TokenType,
        addresses: &[String],
    ) -> Result<Balances, anyhow::Error> {
        match token_type {
            TokenType::Native => {
                let multicall = Multicall::eth_balances(addresses);
                let call = multicall.aggregate(self.contract.clone(), self.contract);
                let result = call.dispatch(client, &self.rpc_url).await?;
                self.rpc_url.zeroize();
                Balances::from_str(&result)
            }
            TokenType::Fungible { address } => {
                let multicall = Multicall::erc20_balances(addresses);
                let call = multicall.aggregate(self.contract, address);
                let result = call.dispatch(client, &self.rpc_url).await?;
                self.rpc_url.zeroize();
                Balances::from_str(&result)
            }
            _ => todo!()
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
