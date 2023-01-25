#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]

pub mod evm;

use async_trait::async_trait;
use ethereum_types::U256;
use rusty_gate_common::TokenType;

#[async_trait]
pub trait BalanceQuerier {
    type Error;
    type Address;

    async fn get_balance_for_many(
        &self,
        token_type: TokenType,
        addresses: &[Self::Address],
    ) -> Result<Vec<U256>, Self::Error>;
    async fn get_balance_for_one(
        &self,
        token_type: TokenType,
        address: Self::Address,
    ) -> Result<U256, Self::Error>;
}
