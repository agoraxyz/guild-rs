use crate::Requirement;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::marker::{Send, Sync};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Free;

#[async_trait]
impl Requirement for Free {
    type Error = ();
    type Identity = Box<dyn Send + Sync>;

    async fn check_for_many(
        &self,
        identities: &[Self::Identity],
    ) -> Result<Vec<bool>, Self::Error> {
        Ok(identities.iter().map(|_| true).collect())
    }

    async fn check(&self, _identity: Self::Identity) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use super::{Free, Requirement};

    #[tokio::test]
    async fn free_requirement_check() {
        let req = Free;

        assert!(req.check(Box::new(69)).await.unwrap());
    }
}
