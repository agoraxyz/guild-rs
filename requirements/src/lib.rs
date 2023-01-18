use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use ethereum_types::{Address, U256};

mod variants;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum Identity {
    EvmAddress(Address),
    SolAccount(String),
    Telegram,
    Discord,
}

pub struct User {
    pub identities: Vec<Identity>,
}

#[derive(Error, Debug)]
pub enum RequirementError {
    #[error("{0}")]
    Other(String),
}

#[async_trait]
pub trait Requirement {
    type Error;

    async fn check_for_many(&self, users: &[User]) -> Result<Vec<bool>, Self::Error>;
    async fn check(&self, user: User) -> Result<bool, Self::Error>;
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
