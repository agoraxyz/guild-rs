#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]

use async_trait::async_trait;
use ethereum_types::U256;
use rusty_gate_common::TokenType;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BalanceError {
    #[error("{0}")]
    Other(String),
}

#[async_trait]
pub trait BalanceQuerier {
    type Address;

    async fn get_balance_for_many(
        &self,
        token_type: TokenType,
        addresses: &[Self::Address],
    ) -> Result<Vec<U256>, BalanceError>;
    async fn get_balance_for_one(
        &self,
        token_type: TokenType,
        address: Self::Address,
    ) -> Result<U256, BalanceError>;
}
