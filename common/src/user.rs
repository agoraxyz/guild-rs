#[cfg(any(feature = "frontend", feature = "test"))]
use primitive_types::H160 as Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(any(feature = "frontend", feature = "test"))]
#[derive(Debug)]
pub enum Identity {
    EvmAddress(Address),
    SolPubkey(String),
    Twitter(u64),
}

#[cfg(any(feature = "frontend", feature = "test"))]
impl Identity {
    pub fn id(&self) -> String {
        format!("{self:?}")
            .chars()
            .take_while(|&ch| ch != '(')
            .collect::<String>()
            .to_lowercase()
    }

    pub fn inner(&self) -> String {
        match self {
            Self::EvmAddress(address) => format!("{address:#x}"),
            Self::SolPubkey(pubkey) => pubkey.to_string(),
            Self::Twitter(id) => format!("{id}"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub id: u64,
    pub identities: HashMap<String, Vec<String>>,
}

impl User {
    #[cfg(any(feature = "frontend", feature = "test"))]
    pub fn new(id: u64) -> Self {
        Self {
            id,
            identities: HashMap::new(),
        }
    }

    #[cfg(any(feature = "frontend", feature = "test"))]
    pub fn add_identity(self, identity: Identity) -> Self {
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

#[cfg(all(test, any(feature = "frontend", feature = "test")))]
mod test {
    use super::Identity;
    use primitive_types::H160 as Address;
    use std::str::FromStr;

    #[test]
    fn identity_test() {
        let twitter = Identity::Twitter(69420);

        assert_eq!(twitter.id(), "twitter");
        assert_eq!(twitter.inner(), "69420");

        let evm_address = Identity::EvmAddress(
            Address::from_str("0xe43878ce78934fe8007748ff481f03b8ee3b97de").unwrap(),
        );

        assert_eq!(evm_address.id(), "evmaddress");
        assert_eq!(
            evm_address.inner(),
            "0xe43878ce78934fe8007748ff481f03b8ee3b97de"
        );
    }
}
