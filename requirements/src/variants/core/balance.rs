use crate::{Requirement, RequirementError};
use async_trait::async_trait;
use ethereum_types::{Address, U256};
use rusty_gate_common::TokenType;
use rusty_gate_providers::{
    evm::{EvmChain, RpcProvider},
    BalanceQuerier,
};
use serde::{Deserialize, Serialize};

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
    pub chain: EvmChain,
    pub token_type: TokenType,
    pub relation: Relation<U256>,
}

#[async_trait]
impl Requirement for Balance {
    type Error = RequirementError;
    type Identity = Address;
    type Client = reqwest::Client;

    async fn check_for_many(
        &self,
        client: &Self::Client,
        identities: &[Self::Identity],
    ) -> Result<Vec<bool>, Self::Error> {
        let balances: Vec<U256> = RpcProvider
            .get_balance_for_many(client, self.chain, self.token_type, identities)
            .await
            .map_err(|err| RequirementError::Other(err.to_string()))?;

        Ok(balances
            .iter()
            .map(|balance| self.relation.assert(balance))
            .collect())
    }

    async fn check(
        &self,
        client: &Self::Client,
        user: Self::Identity,
    ) -> Result<bool, Self::Error> {
        self.check_for_many(client, &[user]).await.map(|res| res[0])
    }
}

#[cfg(test)]
mod test {
    use super::{Balance, Relation, Requirement, TokenType, U256};
    use rusty_gate_common::address;
    use rusty_gate_providers::evm::EvmChain;

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
            chain: EvmChain::Ethereum,
            token_type: TokenType::NonFungible {
                address: address!("0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85"),
                id: None,
            },
            relation: Relation::GreaterThan(U256::from(0)),
        };

        let client = reqwest::Client::new();

        assert!(req
            .check(
                &client,
                address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")
            )
            .await
            .unwrap());

        assert!(req
            .check(
                &client,
                address!("0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3")
            )
            .await
            .unwrap());
    }
}
