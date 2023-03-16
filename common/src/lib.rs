#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use core::ops::{Range, RangeInclusive};
use primitive_types::H160 as Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Scalar = f64;

#[derive(Debug)]
pub enum RequirementType {
    EvmBalance,
}

#[derive(Debug)]
pub enum Chain {
    Ethereum,
    Polygon,
    Gnosis,
    Bsc,
    Goerli,
    Arbitrum,
}

impl ToString for Chain {
    fn to_string(&self) -> String {
        format!("{self:?}").to_lowercase()
    }
}

#[derive(Debug)]
pub enum Identity {
    EvmAddress(Address),
    SolPubkey(String),
    Twitter(u64),
}

impl Identity {
    pub fn id(&self) -> String {
        format!("{self:?}")
            .chars()
            .take_while(|&ch| ch != '(')
            .collect::<String>()
            .to_lowercase()
    }

    pub fn inner(&self) -> String {
        match self {
            Self::EvmAddress(address) => format!("{address:#x}"),
            Self::SolPubkey(pubkey) => pubkey.to_string(),
            Self::Twitter(id) => format!("{id}"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub id: u64,
    pub identities: HashMap<String, Vec<String>>,
}

impl User {
    #[cfg(any(feature = "frontend", feature = "test"))]
    pub fn new(id: u64) -> Self {
        Self {
            id,
            identities: HashMap::new(),
        }
    }

    #[cfg(any(feature = "frontend", feature = "test"))]
    pub fn add_identity(self, identity: Identity) -> Self {
        let id_type = identity.id();
        let mut identities = self.identities;
        let mut vec: Vec<String> = identities
            .get(&id_type)
            .map(|identities| identities.to_vec())
            .unwrap_or_default();

        vec.push(identity.inner());

        identities.insert(id_type, vec);

        Self {
            id: self.id,
            identities,
        }
    }

    pub fn get_identities(&self, id_type: &str) -> Vec<String> {
        self.identities.get(id_type).cloned().unwrap_or_default()
    }
}

#[derive(Debug)]
pub enum TokenType {
    Native,
    Fungible { address: String },
    NonFungible { address: String, id: Option<String> },
    Special { address: String, id: Option<String> },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Relation {
    EqualTo(Scalar),
    GreaterThan(Scalar),
    GreaterOrEqualTo(Scalar),
    LessThan(Scalar),
    LessOrEqualTo(Scalar),
    Between(Range<Scalar>),
    BetweenInclusive(RangeInclusive<Scalar>),
}

impl Relation {
    pub fn assert(&self, x: &Scalar) -> bool {
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

#[cfg(test)]
mod test {
    use super::{Address, Identity, Relation};
    use shiba as _;
    use std::str::FromStr;

    #[test]
    fn identity_test() {
        let twitter = Identity::Twitter(69420);

        assert_eq!(twitter.id(), "twitter");
        assert_eq!(twitter.inner(), "69420");

        let evm_address = Identity::EvmAddress(
            Address::from_str("0xe43878ce78934fe8007748ff481f03b8ee3b97de").unwrap(),
        );

        assert_eq!(evm_address.id(), "evmaddress");
        assert_eq!(
            evm_address.inner(),
            "0xe43878ce78934fe8007748ff481f03b8ee3b97de"
        );
    }

    #[test]
    fn relations() {
        assert!(Relation::EqualTo(0.0).assert(&0.0));
        assert!(!Relation::EqualTo(10.0).assert(&2.0));
        assert!(!Relation::EqualTo(420.0).assert(&421.0));

        assert!(!Relation::GreaterThan(10.0).assert(&3.0));
        assert!(!Relation::GreaterThan(10.0).assert(&10.0));
        assert!(Relation::GreaterThan(10.0).assert(&20.0));

        assert!(Relation::GreaterOrEqualTo(23.0).assert(&42.0));
        assert!(Relation::GreaterOrEqualTo(23.0).assert(&23.0));
        assert!(!Relation::GreaterOrEqualTo(23.0).assert(&14.0));

        assert!(Relation::LessThan(23.0).assert(&1.0));
        assert!(!Relation::LessThan(23.0).assert(&23.0));
        assert!(!Relation::LessThan(23.0).assert(&42.0));

        assert!(Relation::LessOrEqualTo(23.0).assert(&1.0));
        assert!(Relation::LessOrEqualTo(23.0).assert(&23.0));
        assert!(!Relation::LessOrEqualTo(23.0).assert(&42.0));

        assert!(!Relation::Between(0.0..100.0).assert(&230.0));
        assert!(!Relation::Between(50.0..100.0).assert(&15.0));
        assert!(!Relation::Between(50.0..100.0).assert(&100.0));
        assert!(Relation::Between(50.0..100.0).assert(&77.0));
        assert!(Relation::Between(50.0..100.0).assert(&50.0));

        assert!(!Relation::BetweenInclusive(0.0..=100.0).assert(&230.0));
        assert!(!Relation::BetweenInclusive(50.0..=100.0).assert(&15.0));
        assert!(Relation::BetweenInclusive(50.0..=100.0).assert(&100.0));
        assert!(Relation::BetweenInclusive(50.0..=100.0).assert(&77.0));
        assert!(Relation::BetweenInclusive(50.0..=100.0).assert(&50.0));
    }
}
