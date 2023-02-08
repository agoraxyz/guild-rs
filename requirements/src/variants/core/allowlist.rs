use crate::Requirement;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    cmp::PartialEq,
    marker::{Send, Sync},
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AllowList<T> {
    pub identities: Vec<T>,
}

#[async_trait]
impl<T> Requirement for AllowList<T>
where
    T: Sync + Send + PartialEq,
{
    type Error = ();
    type Identity = T;
    type Client = ();

    async fn check_for_many(
        &self,
        _client: &Self::Client,
        identities: &[Self::Identity],
    ) -> Result<Vec<bool>, Self::Error> {
        Ok(identities
            .iter()
            .map(|identity| self.identities.contains(identity))
            .collect())
    }

    async fn check(
        &self,
        client: &Self::Client,
        identity: Self::Identity,
    ) -> Result<bool, Self::Error> {
        self.check_for_many(client, &[identity])
            .await
            .map(|res| res[0])
    }
}

#[cfg(test)]
mod test {
    use super::{AllowList, Requirement};

    #[tokio::test]
    async fn allowlist_requirement_check() {
        let req = AllowList {
            identities: vec![69, 420],
        };

        assert!(req.check(&(), 69).await.unwrap());

        assert!(!req.check(&(), 13).await.unwrap());
    }
}
