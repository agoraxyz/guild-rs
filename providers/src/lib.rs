#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use async_trait::async_trait;
use rusty_gate_common::TokenType;
#[cfg(feature = "nomock")]
use tokio as _;

pub mod evm;

#[async_trait]
pub trait BalanceQuerier {
    type Error;
    type Chain;
    type Address;
    type Id;
    type Balance;

    async fn get_balance(
        &self,
        client: &reqwest::Client,
        chain: Self::Chain,
        token_type: TokenType<Self::Address, Self::Id>,
        address: Self::Address,
    ) -> Result<Self::Balance, Self::Error>;
    async fn get_balance_batch(
        &self,
        client: &reqwest::Client,
        chain: Self::Chain,
        token_type: TokenType<Self::Address, Self::Id>,
        addresses: &[Self::Address],
    ) -> Result<Vec<Self::Balance>, Self::Error>;
}
