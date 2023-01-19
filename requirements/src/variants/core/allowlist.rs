use crate::{Identity, Requirement, RequirementError, User};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AllowList {
    pub identities: Vec<Identity>,
}

#[async_trait]
impl Requirement for AllowList {
    type Error = RequirementError;

    async fn check_for_many(&self, users: &[User]) -> Result<Vec<bool>, Self::Error> {
        Ok(users
            .iter()
            .map(|user| {
                user.identities
                    .iter()
                    .any(|identity| self.identities.contains(identity))
            })
            .collect())
    }

    async fn check(&self, user: User) -> Result<bool, Self::Error> {
        self.check_for_many(&[user]).await.map(|res| res[0])
    }
}

#[cfg(test)]
mod test {
    use super::{AllowList, Requirement, User};
    use crate::evm_addr;

    #[tokio::test]
    async fn allowlist_requirement_check() {
        let req = AllowList {
            identities: vec![
                evm_addr!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"),
                evm_addr!("0x20CC54c7ebc5f43b74866D839b4BD5c01BB23503"),
            ],
        };

        assert!(req
            .check(User {
                identities: vec![evm_addr!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")]
            })
            .await
            .unwrap());

        assert!(!req
            .check(User {
                identities: vec![evm_addr!("0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3")]
            })
            .await
            .unwrap());
    }
}
