#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use async_trait::async_trait;
use futures::future::join_all;
use guild_common::User;
use guild_requirements::{AllowList, Requirement};
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

pub struct Role {
    pub id: String,
    pub filter: Option<AllowList<String>>,
    pub logic: String,
    pub requirements: Option<Vec<Requirement>>,
}

#[derive(Error, Debug)]
pub enum RoleError {
    #[error("Missing requirements")]
    InvalidRole,
    #[error(transparent)]
    Requiem(#[from] requiem::ParseError),
}

#[async_trait]
trait Checkable {
    async fn check(&self, client: &reqwest::Client, user: &User) -> Result<bool, RoleError>;
    async fn check_batch(
        &self,
        client: &reqwest::Client,
        users: &[User],
    ) -> Result<Vec<bool>, RoleError>;
}

#[async_trait]
impl Checkable for Role {
    async fn check(&self, client: &reqwest::Client, user: &User) -> Result<bool, RoleError> {
        self.check_batch(client, &[user.clone()])
            .await
            .map(|accesses| accesses[0])
    }

    async fn check_batch(
        &self,
        client: &reqwest::Client,
        users: &[User],
    ) -> Result<Vec<bool>, RoleError> {
        let accesses_per_req = join_all(
            self.requirements
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .cloned()
                .map(|req| async move {
                    let identities_with_ids: Vec<(u64, String)> = users
                        .iter()
                        .flat_map(|user| {
                            user.identities(&req.identity_id)
                                .unwrap_or(&vec![])
                                .iter()
                                .cloned()
                                .map(|identity| (user.id, identity))
                                .collect::<Vec<_>>()
                        })
                        .collect();

                    let identities: Vec<String> = identities_with_ids
                        .iter()
                        .cloned()
                        .map(|(_, identity)| identity)
                        .collect();

                    let accesses = req.check_batch(client, &identities).await.unwrap();

                    let id_accesses = identities_with_ids
                        .iter()
                        .zip(accesses.iter())
                        .map(|((user_id, _), access)| (*user_id, *access))
                        .collect::<Vec<(u64, bool)>>();

                    users
                        .iter()
                        .map(|user| {
                            id_accesses
                                .iter()
                                .filter_map(
                                    |(i, access)| if &user.id == i { Some(access) } else { None },
                                )
                                .cloned()
                                .reduce(|a, b| a || b)
                                .unwrap_or_default()
                        })
                        .collect()
                }),
        )
        .await;

        let rotated: Vec<Vec<bool>> = (0..users.len())
            .map(|i| {
                accesses_per_req
                    .iter()
                    .cloned()
                    .map(|row: Vec<bool>| row[i])
                    .collect()
            })
            .collect();

        let tree = requiem::LogicTree::from_str(&self.logic)?;

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
            .collect::<Vec<_>>();

        let res = match self.filter.as_ref() {
            Some(filter) => {
                let list = users
                    .iter()
                    .map(|user| {
                        user.identities("evm_address")
                            .unwrap_or(&vec![])
                            .iter()
                            .any(|address| filter.check(address))
                    })
                    .collect::<Vec<_>>();

                res.iter()
                    .enumerate()
                    .map(|(idx, item)| *item && list[idx])
                    .collect()
            }
            _ => res,
        };

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use primitive_types as _;
    use tokio as _;

    #[tokio::test]
    #[cfg(feature = "identity")]
    async fn role_check() {
        use crate::{Checkable, Role};
        use guild_common::{
            identity::{Identity::EvmAddress, UserBuilder},
            Chain, Relation, TokenType,
        };
        use guild_requirements::{AllowList, Balance, Requirement};
        use primitive_types::H160 as Address;
        use std::str::FromStr;

        let allowlist = AllowList {
            deny_list: false,
            list: vec![
                "0xe43878ce78934fe8007748ff481f03b8ee3b97de".to_string(),
                "0x14ddfe8ea7ffc338015627d160ccaf99e8f16dd3".to_string(),
            ],
        };

        let denylist = AllowList {
            deny_list: true,
            list: vec![
                "0x283d678711daa088640c86a1ad3f12c00ec1252e".to_string(),
                "0x20CC54c7ebc5f43b74866d839b4bd5c01bb23503".to_string(),
            ],
        };

        let balance_check = Balance {
            chain: Chain::Ethereum,
            token_type: TokenType::NonFungible {
                address: "0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85".to_string(),
                id: None,
            },
            relation: Relation::GreaterThan(0.0),
        };

        let req = Requirement::from(balance_check);

        let role1 = Role {
            id: "69".to_string(),
            logic: "0".to_string(),
            filter: Some(allowlist),
            requirements: Some(vec![req.clone()]),
        };

        let role2 = Role {
            id: "69".to_string(),
            logic: "0".to_string(),
            filter: Some(denylist),
            requirements: Some(vec![req]),
        };

        let user1 = UserBuilder::new(69)
            .add_identity(EvmAddress(
                Address::from_str("0xe43878ce78934fe8007748ff481f03b8ee3b97de").unwrap(),
            ))
            .build();

        let user2 = UserBuilder::new(420)
            .add_identity(EvmAddress(
                Address::from_str("0x283d678711daa088640c86a1ad3f12c00ec1252e").unwrap(),
            ))
            .build();

        let users = vec![user1, user2];

        let client = reqwest::Client::new();

        assert_eq!(
            role1.check_batch(&client, &users).await.unwrap(),
            &[true, false]
        );
        assert_eq!(
            role2.check_batch(&client, &users).await.unwrap(),
            &[true, false]
        );
    }
}
