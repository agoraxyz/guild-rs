use crate::{Address, Identity, Requirement, RequirementError, User, U256};
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
pub enum Relation<T> {
    EqualTo(T),
    GreaterThan(T),
    GreaterOrEqualTo(T),
    LessThan(T),
    LessOrEqualTo(T),
    Between(core::ops::Range<T>),
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
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Balance {
    // TODO: use the Chain type from the providers crate
    // https://github.com/agoraxyz/requirement-engine-v2/issues/6#issue-1530872075
    pub chain: u64,
    pub token_type: TokenType,
    pub relation: Relation<U256>,
}

#[async_trait]
impl Requirement for Balance {
    type Error = RequirementError;

    async fn check_for_many(&self, users: &[User]) -> Result<Vec<bool>, Self::Error> {
        let identities: Vec<Identity> = users
            .iter()
            .flat_map(|user| user.identities.clone())
            .collect();

        // TODO: use providers to query balance
        // https://github.com/agoraxyz/requirement-engine-v2/issues/6#issue-1530872075
        let balances: Vec<U256> = identities
            .iter()
            .map(|identity| match identity {
                Identity::EvmAddress(_) | Identity::SolAccount(_) => U256::from(69),
                _ => U256::from(0),
            })
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
    use super::{Balance, Identity, Relation, Requirement, TokenType, User, U256};
    use crate::evm_addr;

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
    }

    #[tokio::test]
    async fn balance_requirement_check() {
        let req = Balance {
            chain: 69,
            token_type: TokenType::Coin,
            relation: Relation::GreaterThan(U256::from(0)),
        };

        assert!(req
            .check(User {
                identities: vec![evm_addr!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")]
            })
            .await
            .unwrap());

        assert!(req
            .check(User {
                identities: vec![evm_addr!("0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3")]
            })
            .await
            .unwrap());

        assert!(!req
            .check(User {
                identities: vec![Identity::Telegram(69)]
            })
            .await
            .unwrap());
    }
}
