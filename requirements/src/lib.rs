use async_trait::async_trait;
use thiserror::Error;

pub use web3::types::{Address, U256};

mod variants;

pub struct User {
    pub addresses: Vec<Address>,
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
macro_rules! address {
    ($addr:expr) => {{
        use std::str::FromStr;
        web3::types::Address::from_str($addr).expect(&format!("Invalid address {}", $addr))
    }};
}
