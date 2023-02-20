#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use async_trait::async_trait;
use ethereum_types::Address;
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
pub enum TokenType<T, U> {
    Native,
    Fungible { address: T },
    NonFungible { address: T, id: Option<U> },
    Special { address: T, id: Option<U> },
}

#[derive(Error, Debug)]
pub enum RequirementError {
    #[error("{0}")]
    Other(String),
}

pub trait Requirement {
    type Error;
    type VerificationData;

    fn verify(&self, vd: &Self::VerificationData) -> bool;
    fn verify_batch(&self, vd: &[Self::VerificationData]) -> Vec<bool>;
}

#[async_trait]
pub trait VerificationData {
    type Error;
    type Identity;
    type Client;
    type Res;

    async fn retrieve(
        &self,
        client: &Self::Client,
        identity: &Self::Identity,
    ) -> Result<Self::Res, Self::Error>;
    async fn retrieve_batch(
        &self,
        client: &Self::Client,
        identities: &[Self::Identity],
    ) -> Result<Vec<Self::Res>, Self::Error>;
}

#[macro_export]
macro_rules! address {
    ($addr:expr) => {{
        use std::str::FromStr;
        ethereum_types::H160::from_str($addr).expect(&format!("Invalid address {}", $addr))
    }};
}
