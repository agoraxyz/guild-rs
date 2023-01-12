use async_trait::async_trait;
use thiserror::Error;

pub use web3::types::Address;

mod general;

pub struct User {
    pub addresses: Vec<Address>,
}

#[derive(Error, Debug)]
pub enum RequirementError {
    #[error("{0}")]
    Other(String),
}

type Error = RequirementError;

#[async_trait]
pub trait Requirement {
    async fn check_for_many(&self, users: &[User]) -> Result<Vec<bool>, Error>;
    async fn check(&self, user: User) -> Result<bool, Error>;
}
