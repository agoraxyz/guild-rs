use crate::{Error, Requirement, User};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FreeRequirement;

#[async_trait]
impl Requirement for FreeRequirement {
    async fn check_for_many(&self, users: &[User]) -> Result<Vec<bool>, Error> {
        Ok(users.iter().map(|_| true).collect())
    }

    async fn check(&self, user: User) -> Result<bool, Error> {
        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use super::{FreeRequirement, Requirement, User};

    #[tokio::test]
    async fn free_requirement_check() {
        let req = FreeRequirement;

        assert!(req.check(User { addresses: vec![] }).await.unwrap());
    }
}
