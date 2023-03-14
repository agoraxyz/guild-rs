#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use async_trait::async_trait;
pub use evm::{BalancyError, RpcError};
use guild_common::{Scalar, TokenType};
#[cfg(feature = "nomock")]
use tokio as _;

pub mod evm;

#[async_trait]
pub trait BalanceQuerier {
    type Error;

    async fn get_balance(
        &self,
        client: &reqwest::Client,
        chain: &str,
        token_type: &TokenType,
        address: &str,
    ) -> Result<Scalar, Self::Error>;
    async fn get_balance_batch(
        &self,
        client: &reqwest::Client,
        chain: &str,
        token_type: &TokenType,
        addresses: &[String],
    ) -> Result<Vec<Scalar>, Self::Error>;
}
