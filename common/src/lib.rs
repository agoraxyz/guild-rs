#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use async_trait::async_trait;
use core::ops::{Range, RangeInclusive};
use primitive_types::H160 as Address;
use serde::{Deserialize, Serialize};

pub type Scalar = f64;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum Identity {
    EvmAddress(Address),
    SolAccount(String),
    Telegram(u64),
    Discord(u64),
}

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub id: u64,
    pub identities: Vec<Identity>,
}

pub struct Role {
    pub name: String,
    pub logic: String,
    pub requirements: Vec<Box<dyn Send + Sync + std::any::Any>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum TokenType<T, U> {
    Native,
    Fungible { address: T },
    NonFungible { address: T, id: Option<U> },
    Special { address: T, id: Option<U> },
}

pub trait Requirement {
    type VerificationData;

    fn verify(&self, vd: &Self::VerificationData) -> bool;
    fn verify_batch(&self, vd: &[Self::VerificationData]) -> Vec<bool>;
}

#[async_trait]
pub trait Retrievable {
    type Error;
    type Identity;
    type Client;

    async fn retrieve(
        &self,
        client: &Self::Client,
        identity: &Self::Identity,
    ) -> Result<Scalar, Self::Error>;
    async fn retrieve_batch(
        &self,
        client: &Self::Client,
        identities: &[Self::Identity],
    ) -> Result<Vec<Scalar>, Self::Error>;
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Relation<T> {
    EqualTo(T),
    GreaterThan(T),
    GreaterOrEqualTo(T),
    LessThan(T),
    LessOrEqualTo(T),
    Between(Range<T>),
    BetweenInclusive(RangeInclusive<T>),
}

impl<T: PartialEq + PartialOrd> Relation<T> {
    pub fn assert(&self, x: &T) -> bool {
        match self {
            Relation::EqualTo(a) => x == a,
            Relation::GreaterThan(a) => x > a,
            Relation::GreaterOrEqualTo(a) => x >= a,
            Relation::LessThan(a) => x < a,
            Relation::LessOrEqualTo(a) => x <= a,
            Relation::Between(range) => range.contains(x),
            Relation::BetweenInclusive(range) => range.contains(x),
        }
    }
}

#[macro_export]
macro_rules! address {
    ($addr:expr) => {{
        use std::str::FromStr;
        primitive_types::H160::from_str($addr).expect(&format!("Invalid address {}", $addr))
    }};
}

#[cfg(test)]
mod test {
    use super::Relation;
    use shiba as _;

    #[test]
    fn relations() {
        assert!(Relation::<u32>::EqualTo(0).assert(&0));
        assert!(!Relation::<u32>::EqualTo(10).assert(&2));
        assert!(!Relation::<u32>::EqualTo(420).assert(&421));

        assert!(!Relation::<u32>::GreaterThan(10).assert(&3));
        assert!(!Relation::<u32>::GreaterThan(10).assert(&10));
        assert!(Relation::<u32>::GreaterThan(10).assert(&20));

        assert!(Relation::<u32>::GreaterOrEqualTo(23).assert(&42));
        assert!(Relation::<u32>::GreaterOrEqualTo(23).assert(&23));
        assert!(!Relation::<u32>::GreaterOrEqualTo(23).assert(&14));

        assert!(Relation::<u32>::LessThan(23).assert(&1));
        assert!(!Relation::<u32>::LessThan(23).assert(&23));
        assert!(!Relation::<u32>::LessThan(23).assert(&42));

        assert!(Relation::<u32>::LessOrEqualTo(23).assert(&1));
        assert!(Relation::<u32>::LessOrEqualTo(23).assert(&23));
        assert!(!Relation::<u32>::LessOrEqualTo(23).assert(&42));

        assert!(!Relation::<u32>::Between(0..100).assert(&230));
        assert!(!Relation::<u32>::Between(50..100).assert(&15));
        assert!(!Relation::<u32>::Between(50..100).assert(&100));
        assert!(Relation::<u32>::Between(50..100).assert(&77));
        assert!(Relation::<u32>::Between(50..100).assert(&50));

        assert!(!Relation::<u32>::BetweenInclusive(0..=100).assert(&230));
        assert!(!Relation::<u32>::BetweenInclusive(50..=100).assert(&15));
        assert!(Relation::<u32>::BetweenInclusive(50..=100).assert(&100));
        assert!(Relation::<u32>::BetweenInclusive(50..=100).assert(&77));
        assert!(Relation::<u32>::BetweenInclusive(50..=100).assert(&50));
    }
}
