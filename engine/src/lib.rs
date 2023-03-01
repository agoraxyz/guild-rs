#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use async_trait::async_trait;
use futures::future::join_all;
use guild_common::{Identity, Requirement, Retrievable, Role, User};
use guild_requirements::{AllowList, Balance, Free};
use primitive_types::{H160 as Address, U256};
use std::{collections::HashMap, str::FromStr};

#[async_trait]
trait Checkable {
    async fn check(&self, user: &User) -> Result<bool, ()>;
    async fn check_batch(&self, users: &[User]) -> Result<Vec<bool>, ()>;
}

#[async_trait]
impl Checkable for Role {
    async fn check(&self, user: &User) -> Result<bool, ()> {
        self.check_batch(&[user.clone()])
            .await
            .map(|accesses| accesses[0])
    }

    async fn check_batch(&self, users: &[User]) -> Result<Vec<bool>, ()> {
        let users_count = users.len();
        let ids: Vec<u64> = users.iter().map(|user| user.id).collect();
        let id_addresses: Vec<(u64, Address)> = users
            .iter()
            .flat_map(|user| {
                user.identities
                    .iter()
                    .filter_map(|identity| match identity {
                        Identity::EvmAddress(address) => Some((user.id, *address)),
                        _ => None,
                    })
            })
            .collect();
        let addresses: Vec<Address> = id_addresses.iter().map(|(_, address)| *address).collect();

        let reduce_accesses = |accesses: &[bool]| -> Vec<bool> {
            let id_accesses = id_addresses
                .iter()
                .zip(accesses.iter())
                .map(|((user_id, _), access)| (*user_id, *access))
                .collect::<Vec<(u64, bool)>>();

            ids.iter()
                .map(|id| {
                    id_accesses
                        .iter()
                        .filter_map(|(i, access)| if id == i { Some(access) } else { None })
                        .cloned()
                        .reduce(|a, b| a || b)
                        .unwrap_or_default()
                })
                .collect()
        };

        let accesses_per_req = join_all(self.requirements.iter().map(|req| async {
            if let Some(free) = req.downcast_ref::<Free>() {
                free.verify_batch(&vec![(); users_count])
            } else if let Some(allowlist) = req.downcast_ref::<AllowList<Address>>() {
                reduce_accesses(&allowlist.verify_batch(&addresses))
            } else if let Some(balance_check) = req.downcast_ref::<Balance<Address, U256>>() {
                let balances = balance_check
                    .retrieve_batch(&reqwest::Client::new(), &addresses)
                    .await
                    .unwrap();

                reduce_accesses(&balance_check.verify_batch(&balances))
            } else {
                vec![false; users_count]
            }
        }))
        .await;

        let rotated: Vec<Vec<_>> = (0..users_count)
            .map(|i| accesses_per_req.iter().map(|row| row[i]).collect())
            .collect();

        match requiem::LogicTree::from_str(&self.logic) {
            Ok(tree) => {
                let res = rotated
                    .iter()
                    .map(|accesses| {
                        let terminals: HashMap<_, _> = accesses
                            .iter()
                            .enumerate()
                            .map(|(i, &a)| (i as u32, a))
                            .collect();

                        tree.evaluate(&terminals).unwrap_or(false)
                    })
                    .collect();
                Ok(res)
            }
            Err(e) => Ok(vec![false; users_count]),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Checkable;
    use guild_common::{address, Identity, Relation, Role, TokenType, User};
    use guild_providers::evm::EvmChain;
    use guild_requirements::{AllowList, Balance, Free};
    use primitive_types::U256;
    use std::any::Any;

    #[tokio::test]
    async fn role_check() {
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
            relation: Relation::GreaterThan(0.0),
        };

        let requirements: Vec<Box<dyn Send + Sync + Any>> = vec![
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

        let user1 = User {
            id: 69,
            identities: vec![Identity::EvmAddress(address!(
                "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"
            ))],
        };

        let user2 = User {
            id: 420,
            identities: vec![Identity::EvmAddress(address!(
                "0x283d678711daa088640c86a1ad3f12c00ec1252e"
            ))],
        };

        assert_eq!(
            role.check_batch(&[user1, user2]).await.unwrap(),
            &[true, false]
        );
    }
}
