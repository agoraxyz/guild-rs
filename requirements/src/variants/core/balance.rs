use crate::{Requirement, RequirementError};
use async_trait::async_trait;
use guild_common::{Relation, Retrievable, TokenType};
use guild_providers::{
    evm::{EvmChain, Provider},
    BalanceQuerier,
};
use primitive_types::{H160 as Address, U256};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Balance<T, U, V> {
    pub chain: EvmChain,
    pub token_type: TokenType<T, U>,
    pub relation: Relation<V>,
}

impl<T, U, V> Requirement for Balance<T, U, V>
where
    V: PartialEq + PartialOrd,
{
    type VerificationData = V;

    fn verify(&self, vd: &Self::VerificationData) -> bool {
        self.relation.assert(vd)
    }

    fn verify_batch(&self, vd: &[Self::VerificationData]) -> Vec<bool> {
        vd.iter().map(|v| self.verify(v)).collect()
    }
}

#[async_trait]
impl Retrievable for Balance<Address, U256, U256> {
    type Error = RequirementError;
    type Identity = Address;
    type Client = reqwest::Client;
    type Res = U256;

    async fn retrieve(
        &self,
        client: &Self::Client,
        identity: &Self::Identity,
    ) -> Result<Self::Res, Self::Error> {
        Provider
            .get_balance(client, self.chain, self.token_type, *identity)
            .await
            .map_err(|err| RequirementError::Other(err.to_string()))
    }

    async fn retrieve_batch(
        &self,
        client: &Self::Client,
        identities: &[Self::Identity],
    ) -> Result<Vec<Self::Res>, Self::Error> {
        Provider
            .get_balance_batch(client, self.chain, self.token_type, identities)
            .await
            .map_err(|err| RequirementError::Other(err.to_string()))
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "nomock")]
    use super::Retrievable;
    use super::{Balance, Relation, Requirement, TokenType, U256};
    use guild_common::address;
    use guild_providers::evm::EvmChain;

    #[tokio::test]
    async fn balance_requirement_check() {
        let req = Balance {
            chain: EvmChain::Ethereum,
            token_type: TokenType::NonFungible {
                address: address!("0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85"),
                id: None::<U256>,
            },
            relation: Relation::GreaterThan(U256::from(0)),
        };

        #[cfg(feature = "nomock")]
        {
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

        #[cfg(not(feature = "nomock"))]
        {
            let balance_1 = U256::from(69);
            let balance_2 = U256::from(420);

            assert!(req.verify(&balance_1));
            assert!(req.verify(&balance_2));
        }
    }
}
