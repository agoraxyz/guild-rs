#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]

pub mod evm;

use async_trait::async_trait;
use ethereum_types::U256;
use evm::EvmChain;
use rusty_gate_common::TokenType;

#[async_trait]
pub trait BalanceQuerier {
    type Error;
    type Address;

    async fn get_balance_for_many(
        &self,
        client: &reqwest::Client,
        chain: EvmChain,
        token_type: TokenType<Self::Address, U256>,
        addresses: &[Self::Address],
    ) -> Result<Vec<U256>, Self::Error>;
    async fn get_balance_for_one(
        &self,
        client: &reqwest::Client,
        chain: EvmChain,
        token_type: TokenType<Self::Address, U256>,
        address: Self::Address,
    ) -> Result<U256, Self::Error>;
}
