#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]

use async_trait::async_trait;
use ethereum_types::{Address, U256};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum Identity {
    EvmAddress(Address),
    SolAccount(String),
    Telegram(u64),
    Discord(u64),
}

#[derive(Deserialize, Serialize)]
pub struct User {
    pub identities: Vec<Identity>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum TokenType {
    Coin,
    Fungible { address: Address },
    NonFungible { address: Address, id: Option<U256> },
    Special { address: Address, id: Option<U256> },
}

#[derive(Error, Debug)]
pub enum RequirementError {
    #[error("{0}")]
    Other(String),
}

#[async_trait]
pub trait Requirement {
    type Error;
    type Identity;

    async fn check_for_many(&self, identities: &[Self::Identity])
        -> Result<Vec<bool>, Self::Error>;
    async fn check(&self, identity: Self::Identity) -> Result<bool, Self::Error>;
}

#[macro_export]
macro_rules! evm_addr {
    ($addr:expr) => {{
        use std::str::FromStr;
        let addr =
            ethereum_types::H160::from_str($addr).expect(&format!("Invalid address {}", $addr));
        $crate::Identity::EvmAddress(addr)
    }};
}
