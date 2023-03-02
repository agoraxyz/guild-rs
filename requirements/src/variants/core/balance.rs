use async_trait::async_trait;
use guild_common::{Relation, Requirement, Retrievable, Scalar, TokenType};
use guild_providers::{evm::Provider, BalanceQuerier};
use primitive_types::{H160 as Address, U256};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BalanceError {
    #[error(transparent)]
    ProviderError(#[from] guild_providers::RpcError),
    #[error(transparent)]
    BalancyError(#[from] guild_providers::BalancyError),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Balance<T, U> {
    pub chain: String,
    pub token_type: TokenType<T, U>,
    pub relation: Relation,
}

impl<T, U> Requirement for Balance<T, U> {
    type VerificationData = Scalar;

    fn verify(&self, vd: &Self::VerificationData) -> bool {
        self.relation.assert(vd)
    }

    fn verify_batch(&self, vd: &[Self::VerificationData]) -> Vec<bool> {
        vd.iter().map(|v| self.verify(v)).collect()
    }
}

#[async_trait]
impl Retrievable for Balance<Address, U256> {
    type Error = BalanceError;
    type Identity = Address;
    type Client = reqwest::Client;

    async fn retrieve(
        &self,
        client: &Self::Client,
        identity: &Self::Identity,
    ) -> Result<Scalar, Self::Error> {
        let res = Provider
            .get_balance(client, &self.chain, self.token_type, *identity)
            .await?;

        Ok(res)
    }

    async fn retrieve_batch(
        &self,
        client: &Self::Client,
        identities: &[Self::Identity],
    ) -> Result<Vec<Scalar>, Self::Error> {
        let res = Provider
            .get_balance_batch(client, &self.chain, self.token_type, identities)
            .await?;

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "nomock")]
    use super::Retrievable;
    use super::{Balance, Relation, Requirement, TokenType, U256};
    use guild_common::{address, Chain};

    #[tokio::test]
    async fn balance_requirement_check() {
        let req = Balance {
            chain: Chain::Ethereum.to_string(),
            token_type: TokenType::NonFungible {
                address: address!("0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85"),
                id: None::<U256>,
            },
            relation: Relation::GreaterThan(0.0),
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
            let balance_1 = 69.0;
            let balance_2 = 420.0;

            assert!(req.verify(&balance_1));
            assert!(req.verify(&balance_2));
        }
    }
}
