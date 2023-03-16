use core::ops::{Range, RangeInclusive};
use serde::{Deserialize, Serialize};

pub type Scalar = f64;

#[derive(Debug)]
pub enum RequirementType {
    EvmBalance,
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
