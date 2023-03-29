use core::ops::{Range, RangeInclusive};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub enum RequirementType {
    EvmBalance,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Requirement {
    pub id: String,
    pub typ: RequirementType,
    pub metadata: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RequirementResult {
    Ok(Vec<bool>),
    Err(String),
}

impl fmt::Debug for RequirementType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match self {
            Self::EvmBalance => "evm_balance",
        };

        write!(f, "{res}")
    }
}

impl fmt::Display for RequirementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum TokenType {
    Native,
    Fungible { address: String },
    NonFungible { address: String, id: Option<String> },
    Special { address: String, id: Option<String> },
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub enum Relation<T> {
    EqualTo(T),
    GreaterThan(T),
    GreaterOrEqualTo(T),
    LessThan(T),
    LessOrEqualTo(T),
    Between(Range<T>),
    BetweenInclusive(RangeInclusive<T>),
}

impl<T> Relation<T>
where
    T: PartialEq + PartialOrd,
{
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

#[cfg(test)]
mod test {
    use super::Relation;
    use shiba as _;

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
