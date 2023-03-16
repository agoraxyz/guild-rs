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
    pub fn add_identity(self, identity: identity::Identity) -> Self {
        let id_type = identity.id();
        let mut identities = self.identities;
        let mut vec: Vec<String> = identities
            .get(&id_type)
            .map(|identities| identities.to_vec())
            .unwrap_or_default();

        vec.push(identity.inner());

        identities.insert(id_type, vec);

        Self {
            id: self.id,
            identities,
        }
    }

    pub fn get_identities(&self, id_type: &str) -> Vec<String> {
        self.identities.get(id_type).cloned().unwrap_or_default()
    }
}
