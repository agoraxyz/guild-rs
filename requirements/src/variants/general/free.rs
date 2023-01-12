use crate::{Requirement, RequirementError, User};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Free;

#[async_trait]
impl Requirement for Free {
    type Error = RequirementError;

    async fn check_for_many(&self, users: &[User]) -> Result<Vec<bool>, Self::Error> {
        Ok(users.iter().map(|_| true).collect())
    }

    async fn check(&self, _user: User) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use super::{Free, Requirement, User};

    #[tokio::test]
    async fn free_requirement_check() {
        let req = Free;

        assert!(req.check(User { addresses: vec![] }).await.unwrap());
    }
}
