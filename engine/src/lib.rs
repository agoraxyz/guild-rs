#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

pub use allowlist::AllowList;
use guild_common::User;
use guild_requirement::Requirement;
use requiem::{LogicTree, ParseError};
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

mod allowlist;

pub struct Role {
    pub id: String,
    pub filter: Option<AllowList<String>>,
    pub logic: String,
    pub requirements: Vec<Requirement>,
}

#[derive(Error, Debug)]
pub enum RoleError {
    #[error("Missing requirements")]
    InvalidRole,
    #[error(transparent)]
    Requiem(#[from] ParseError),
    #[error("{0}")]
    Requirement(String),
}

impl Role {
    pub async fn check(&self, client: &reqwest::Client, user: &User) -> Result<bool, RoleError> {
        self.check_batch(client, &[user.clone()])
            .await
            .map(|accesses| accesses[0])
    }

    pub async fn check_batch(
        &self,
        client: &reqwest::Client,
        users: &[User],
    ) -> Result<Vec<bool>, RoleError> {
        let acc: Vec<_> = self
            .requirements
            .iter()
            .map(|req| req.check(client, users))
            .collect();

        let acc_res: Result<Vec<Vec<bool>>, _> = acc.into_iter().collect();

        let Ok(acc_per_req) = acc_res else {
            return Err(RoleError::Requirement(acc_res.unwrap_err().to_string()))
        };

        let rotated: Vec<Vec<bool>> = (0..users.len())
            .map(|i| {
                acc_per_req
                    .iter()
                    .cloned()
                    .map(|row: Vec<bool>| row[i])
                    .collect()
            })
            .collect();

        let tree = LogicTree::from_str(&self.logic)?;

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

        if let Some(filter) = self.filter.as_ref() {
            let list = users
                .iter()
                .map(|user| {
                    user.identities("evm_address")
                        .unwrap_or(&vec![])
                        .iter()
                        .any(|address| filter.check(address))
                })
                .collect::<Vec<_>>();

            let filtered = res
                .iter()
                .enumerate()
                .map(|(idx, item)| *item && list[idx])
                .collect();

            return Ok(filtered);
        }

        Ok(res)
    }
}

#[cfg(test)]
mod test_import {
    use serde_json as _;
    use tokio as _;
}

#[cfg(all(test, feature = "test-config"))]
mod test {
    use super::{AllowList, Requirement, Role, User};
    use guild_common::{Chain, Relation, RequirementType, TokenType};

    const USERS: &str = r#"[
    {
        "id": 0,
        "identities": {
            "evm_address": ["0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"]
        }
    },
    {
        "id": 1,
        "identities": {
            "evm_address": ["0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3"]
        }
    },
    {
        "id": 2,
        "identities": {
            "evm_address": ["0x283d678711daa088640c86a1ad3f12c00ec1252e"]
        }
    }
    ]"#;

    #[tokio::test]
    async fn role_check() {
        let allowlist = AllowList {
            deny_list: false,
            list: vec![
                "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE".to_string(),
                "0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3".to_string(),
            ],
        };

        let users: Vec<User> = serde_json::from_str(USERS).unwrap();

        let token_type = TokenType::NonFungible {
            address: "0x57f1887a8BF19b14fC0dF6Fd9B2acc9Af147eA85".to_string(),
            id: None,
        };

        let relation = Relation::GreaterThan(0.0);

        let req = Requirement {
            id: "69".to_string(),
            typ: RequirementType::EvmBalance.to_string(),
            config_key: Chain::Ethereum.to_string(),
            metadata: serde_json::to_string(&(token_type, relation)).unwrap(),
        };

        let role = Role {
            id: "420".to_string(),
            logic: "0".to_string(),
            filter: Some(allowlist),
            requirements: vec![req],
        };

        let client = reqwest::Client::new();

        assert_eq!(
            role.check_batch(&client, &users).await.unwrap(),
            &[true, true, false]
        );
    }
}
