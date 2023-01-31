#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]

pub mod evm;

use async_trait::async_trait;
use ethereum_types::U256;
use evm::EvmChain;
use rusty_gate_common::TokenType;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref CLIENT: RwLock<reqwest::Client> =
        RwLock::new(reqwest::Client::new());
}

#[async_trait]
pub trait BalanceQuerier {
    type Error;
    type Address;

    async fn get_balance_for_many(
        &self,
        chain: EvmChain,
        token_type: TokenType,
        addresses: &[Self::Address],
    ) -> Result<Vec<U256>, Self::Error>;
    async fn get_balance_for_one(
        &self,
        chain: EvmChain,
        token_type: TokenType,
        address: Self::Address,
    ) -> Result<U256, Self::Error>;
}
