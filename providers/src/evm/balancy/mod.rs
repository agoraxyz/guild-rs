use crate::{evm::balancy::types::*, BalanceQuerier};
use async_trait::async_trait;
use ethereum_types::U256;
use rusty_gate_common::TokenType;

mod types;

pub struct BalancyProvider;

pub const BALANCY_PROVIDER: BalancyProvider = BalancyProvider {};

#[async_trait]
impl BalanceQuerier for BalancyProvider {
    type Error = BalancyError;
    type Address = ethereum_types::H160;

    async fn get_balance_for_many(
        &self,
        token_type: TokenType,
        addresses: &[Self::Address],
    ) -> Result<Vec<U256>, Self::Error> {
        Ok(vec![])
    }

    async fn get_balance_for_one(
        &self,
        token_type: TokenType,
        address: Self::Address,
    ) -> Result<U256, Self::Error> {
        self.get_balance_for_many(token_type, &vec![address])
            .await
            .map(|res| res[0])
    }
}
