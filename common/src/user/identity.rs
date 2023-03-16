use primitive_types::H160 as Address;

#[derive(Debug)]
pub enum Identity {
    EvmAddress(Address),
    SolPubkey(String),
    TwitterId(u64),
}

impl Identity {
    pub fn id(&self) -> String {
        match self {
            Self::EvmAddress(_) => "evm_address",
            Self::SolPubkey(_) => "sol_pubkey",
            Self::TwitterId(_) => "twitter_id",
        }
        .to_string()
    }

    pub fn inner(&self) -> String {
        match self {
            Self::EvmAddress(address) => format!("{address:#x}"),
            Self::SolPubkey(pubkey) => pubkey.to_string(),
            Self::TwitterId(id) => format!("{id}"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Identity;
    use primitive_types::H160 as Address;
    use std::str::FromStr;

    #[test]
    fn identity_test() {
        let twitter = Identity::TwitterId(69420);

        assert_eq!(twitter.id(), "twitter_id");
        assert_eq!(twitter.inner(), "69420");

        let evm_address = Identity::EvmAddress(
            Address::from_str("0xe43878ce78934fe8007748ff481f03b8ee3b97de").unwrap(),
        );

        assert_eq!(evm_address.id(), "evm_address");
        assert_eq!(
            evm_address.inner(),
            "0xe43878ce78934fe8007748ff481f03b8ee3b97de"
        );
    }
}
