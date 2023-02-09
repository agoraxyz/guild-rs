use crate::{Requirement, RequirementError};
use async_trait::async_trait;
use ethereum_types::{Address, U256};
use rusty_gate_common::{TokenType, VerificationData};
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
pub struct Balance<T, U> {
    pub chain: EvmChain,
    pub token_type: TokenType<T, U>,
    pub relation: Relation<U256>,
}

impl Requirement for Balance<Address, U256> {
    type Error = RequirementError;
    type VerificationData = U256;

    fn verify(&self, vd: &Self::VerificationData) -> bool {
        self.relation.assert(vd)
    }

    fn verify_batch(&self, vd: &[Self::VerificationData]) -> Vec<bool> {
        vd.iter().map(|v| self.verify(v)).collect()
    }
}

#[async_trait]
impl VerificationData for Balance<Address, U256> {
    type Error = RequirementError;
    type Identity = Address;
    type Client = reqwest::Client;
    type Res = U256;

    async fn retrieve(
        &self,
        client: &Self::Client,
        identity: &Self::Identity,
    ) -> Result<Self::Res, Self::Error> {
        self.retrieve_batch(client, &[*identity])
            .await
            .map(|res| res[0])
    }

    async fn retrieve_batch(
        &self,
        client: &Self::Client,
        identities: &[Self::Identity],
    ) -> Result<Vec<Self::Res>, Self::Error> {
        RpcProvider
            .get_balance_for_many(client, self.chain, self.token_type, identities)
            .await
            .map_err(|err| RequirementError::Other(err.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::{Balance, Relation, Requirement, TokenType, VerificationData, U256};
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

        let balance_1 = req
            .retrieve(
                &client,
                &address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"),
            )
            .await
            .unwrap();
        let balance_2 = req
            .retrieve(
                &client,
                &address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"),
            )
            .await
            .unwrap();

        assert!(req.verify(&balance_1));
        assert!(req.verify(&balance_2));
    }
}
