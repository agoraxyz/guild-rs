#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use guild_common as _;
use guild_requirements as _;
use requiem as _;

#[cfg(test)]
mod test {
    use guild_common::{address, Identity, Relation, Requirement, Role, TokenType, User};
    use guild_providers::evm::EvmChain;
    use guild_requirements::{AllowList, Balance, Free};
    use primitive_types::{H160 as Address, U256};
    use std::{any::Any, collections::HashMap, str::FromStr};

    #[test]
    fn role_check() {
        let allowlist = AllowList {
            deny_list: false,
            verification_data: vec![
                address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"),
                address!("0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3"),
            ],
        };

        let denylist = AllowList {
            deny_list: true,
            verification_data: vec![
                address!("0x283d678711daa088640c86a1ad3f12c00ec1252e"),
                address!("0x20CC54c7ebc5f43b74866D839b4BD5c01BB23503"),
            ],
        };

        let balance_check = Balance {
            chain: EvmChain::Ethereum,
            token_type: TokenType::NonFungible {
                address: address!("0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85"),
                id: None::<U256>,
            },
            relation: Relation::GreaterThan(U256::from(0)),
        };

        let requirements: Vec<Box<dyn Any>> = vec![
            Box::new(Free),
            Box::new(allowlist),
            Box::new(denylist),
            Box::new(balance_check),
        ];

        let role = Role {
            name: "Test Role".to_string(),
            logic: "0 AND 1 AND 2 AND 3".to_string(),
            requirements,
        };

        let user = User {
            identities: vec![Identity::EvmAddress(address!(
                "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"
            ))],
        };

        let addresses = user
            .identities
            .iter()
            .filter_map(|identity| match identity {
                Identity::EvmAddress(address) => Some(address),
                _ => None,
            })
            .cloned()
            .collect::<Vec<Address>>();

        let accesses: Vec<bool> = role
            .requirements
            .iter()
            .map(|req| {
                if let Some(free) = req.downcast_ref::<Free>() {
                    free.verify(&())
                } else if let Some(allowlist) = req.downcast_ref::<AllowList<Address>>() {
                    allowlist
                        .verify_batch(&addresses)
                        .iter()
                        .cloned()
                        .reduce(|a, b| a && b)
                        .unwrap_or(false)
                } else if let Some(balance_check) =
                    req.downcast_ref::<Balance<Address, U256, U256>>()
                {
                    balance_check.verify(&U256::from(69))
                } else {
                    false
                }
            })
            .collect();

        let access = match requiem::LogicTree::from_str(&role.logic) {
            Ok(tree) => {
                let mut terminals = HashMap::new();

                for (idx, access) in accesses.iter().enumerate() {
                    terminals.insert(idx as u32, *access);
                }

                tree.evaluate(&terminals).unwrap_or(false)
            }
            Err(_) => false,
        };

        assert!(access);
    }
}
