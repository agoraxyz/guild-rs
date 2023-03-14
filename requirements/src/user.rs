use guild_common::Identity;
use std::collections::HashMap;

#[derive(Debug)]
pub struct User {
    pub identities: HashMap<String, Vec<String>>,
}

impl User {
    pub fn new() -> Self {
        Self {
            identities: HashMap::new(),
        }
    }

    pub fn add_identity(self, id_type: String, identity: String) -> Self {
        let mut identities = self.identities;
        let mut vec: Vec<String> = identities
            .get(&id_type)
            .map(|identities| identities.iter().cloned().collect())
            .unwrap_or_default();

        vec.push(identity);

        identities.insert(id_type, vec);

        Self { identities }
    }

    pub fn get_identities(&self, id_type: &str) -> Vec<String> {
        self.identities.get(id_type).cloned().unwrap_or_default()
    }
}
