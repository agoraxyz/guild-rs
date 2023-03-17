use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "identity")]
pub mod identity;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub id: u64,
    pub identities: HashMap<String, Vec<String>>,
}

impl User {
    pub fn identities(&self, id_type: &str) -> Option<&Vec<String>> {
        self.identities.get(id_type)
    }
}

#[cfg(all(test, feature = "identity"))]
mod test {
    use super::identity::{Identity, UserBuilder};

    #[test]
    fn add_identity_test() {
        let user = UserBuilder::new(69)
            .add_identity(Identity::TwitterId(420))
            .add_identity(Identity::TwitterId(23))
            .build();

        assert_eq!(user.identities("twitter_id").unwrap(), &["420", "23"]);
    }
}
