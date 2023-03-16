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
    #[cfg(any(feature = "identity"))]
    pub fn new(id: u64) -> Self {
        Self {
            id,
            identities: HashMap::new(),
        }
    }

    #[cfg(any(feature = "identity"))]
    pub fn add_identity(&mut self, identity: identity::Identity) -> &mut Self {
        let id_type = identity.id();
        let mut vec: Vec<String> = self
            .identities
            .get(&id_type)
            .map(|identities| identities.to_vec())
            .unwrap_or_default();

        vec.push(identity.inner());

        self.identities.insert(id_type, vec);

        self
    }

    pub fn get_identities(&self, id_type: &str) -> Vec<String> {
        self.identities.get(id_type).cloned().unwrap_or_default()
    }
}

#[cfg(all(test, feature = "identity"))]
mod test {
    use super::{identity::Identity, User};

    #[test]
    fn add_identity_test() {
        let mut user = User::new(69);
        user.add_identity(Identity::TwitterId(420))
            .add_identity(Identity::TwitterId(23));

        dbg!(user);
    }
}
