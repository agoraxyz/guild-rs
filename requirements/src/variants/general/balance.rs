use crate::{Address, Requirement, RequirementError, User, U256};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum TokenType {
    Coin,
    Fungible { address: Address },
    NonFungible { address: Address, id: U256 },
    Special { address: Address, id: U256 },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Relation {
    EqualTo(U256),
    GreaterThan(U256),
    GreaterOrEqualTo(U256),
    LessThan(U256),
    LessOrEqualTo(U256),
    Between(std::ops::Range<U256>),
}

impl Relation {
    pub fn assert(&self, x: &U256) -> bool {
        match self {
            Relation::EqualTo(a) => x == a,
            Relation::GreaterThan(a) => x > a,
            Relation::GreaterOrEqualTo(a) => x >= a,
            Relation::LessThan(a) => x < a,
            Relation::LessOrEqualTo(a) => x <= a,
            Relation::Between(range) => range.contains(x),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BalanceRequirement {
    // TODO: use the Chain type from the providers crate
    // https://github.com/agoraxyz/requirement-engine-v2/issues/6#issue-1530872075
    pub chain: u64,
    pub token_type: TokenType,
    pub relation: Relation,
}

#[async_trait]
impl Requirement for BalanceRequirement {
    type Error = RequirementError;

    async fn check_for_many(&self, users: &[User]) -> Result<Vec<bool>, Self::Error> {
        let addresses: Vec<Address> = users
            .iter()
            .flat_map(|user| user.addresses.clone())
            .collect();

        // TODO: use providers to query balance
        // https://github.com/agoraxyz/requirement-engine-v2/issues/6#issue-1530872075
        let balances: Vec<U256> = addresses
            .iter()
            .map(|_| U256::from_dec_str("69").unwrap())
            .collect();

        // TODO: use the appropriate function of providers
        // https://github.com/agoraxyz/requirement-engine-v2/issues/6#issue-1530872075
        // match self.token_type {
        //     TokenType::Coin => {}
        //     TokenType::Fungible { address } => {}
        //     TokenType::NonFungible { address, id } => {}
        //     TokenType::Special { address, id } => {}
        // }

        Ok(balances
            .iter()
            .map(|balance| self.relation.assert(balance))
            .collect())
    }

    async fn check(&self, user: User) -> Result<bool, Self::Error> {
        self.check_for_many(&[user]).await.map(|res| res[0])
    }
}

#[cfg(test)]
mod test {
    use super::{BalanceRequirement, Relation, Requirement, TokenType, User, U256};
    use crate::address;

    macro_rules! u256 {
        ($num: expr) => {
            U256::from_dec_str(&format!("{}", $num)).unwrap()
        };
    }

    #[test]
    fn relations() {
        assert!(Relation::EqualTo(u256!(0)).assert(&u256!(0)));
        assert!(!Relation::EqualTo(u256!(10)).assert(&u256!(2)));
        assert!(!Relation::EqualTo(u256!(420)).assert(&u256!(421)));

        assert!(!Relation::GreaterThan(u256!(10)).assert(&u256!(3)));
        assert!(!Relation::GreaterThan(u256!(10)).assert(&u256!(10)));
        assert!(Relation::GreaterThan(u256!(10)).assert(&u256!(20)));

        assert!(Relation::GreaterOrEqualTo(u256!(23)).assert(&u256!(42)));
        assert!(Relation::GreaterOrEqualTo(u256!(23)).assert(&u256!(23)));
        assert!(!Relation::GreaterOrEqualTo(u256!(23)).assert(&u256!(14)));

        assert!(Relation::LessThan(u256!(23)).assert(&u256!(1)));
        assert!(!Relation::LessThan(u256!(23)).assert(&u256!(23)));
        assert!(!Relation::LessThan(u256!(23)).assert(&u256!(42)));

        assert!(Relation::LessOrEqualTo(u256!(23)).assert(&u256!(1)));
        assert!(Relation::LessOrEqualTo(u256!(23)).assert(&u256!(23)));
        assert!(!Relation::LessOrEqualTo(u256!(23)).assert(&u256!(42)));

        assert!(!Relation::Between(u256!(0)..u256!(100)).assert(&u256!(230)));
        assert!(!Relation::Between(u256!(50)..u256!(100)).assert(&u256!(15)));
        assert!(!Relation::Between(u256!(50)..u256!(100)).assert(&u256!(100)));
        assert!(Relation::Between(u256!(50)..u256!(100)).assert(&u256!(77)));
        assert!(Relation::Between(u256!(50)..u256!(100)).assert(&u256!(50)));
    }

    #[tokio::test]
    async fn balance_requirement_check() {
        let req = BalanceRequirement {
            chain: 69,
            token_type: TokenType::Coin,
            relation: Relation::GreaterThan(u256!(0)),
        };

        assert!(req
            .check(User {
                addresses: vec![address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")]
            })
            .await
            .unwrap());

        assert!(req
            .check(User {
                addresses: vec![address!("0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3")]
            })
            .await
            .unwrap());
    }
}
