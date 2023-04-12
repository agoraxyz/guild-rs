#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

pub use allowlist::AllowList;
use guild_common::User;
use guild_requirement::{RedisCache, Requirement};
use requiem::{LogicTree, ParseError};
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

mod allowlist;

type AccessMatrix = Vec<Vec<bool>>;

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
    pub async fn check(
        &self,
        redis_cache: &mut RedisCache,
        client: &reqwest::Client,
        user: &User,
    ) -> Result<bool, RoleError> {
        self.check_batch(redis_cache, client, &[user.clone()])
            .await
            .map(|accesses| accesses[0])
    }

    pub async fn check_batch(
        &self,
        redis_cache: &mut RedisCache,
        client: &reqwest::Client,
        users: &[User],
    ) -> Result<Vec<bool>, RoleError> {
        let acc: Vec<_> = self
            .requirements
            .iter()
            .map(|req| req.check(redis_cache, client, users))
            .collect();

        let acc_res: Result<AccessMatrix, _> = acc.into_iter().collect();

        let Ok(acc_per_req) = acc_res else {
            return Err(RoleError::Requirement(acc_res.unwrap_err().to_string()))
        };

        let rotated: AccessMatrix = rotate_matrix(&acc_per_req, users.len());
        let res = evaluate_access_matrix(&rotated, &self.logic)?;

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

fn evaluate_access_matrix(matrix: &AccessMatrix, logic: &str) -> Result<Vec<bool>, ParseError> {
    let tree = LogicTree::from_str(logic)?;

    let res = matrix
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

    Ok(res)
}

fn rotate_matrix(matrix: &AccessMatrix, length: usize) -> AccessMatrix {
    (0..length)
        .map(|i| {
            matrix
                .iter()
                .cloned()
                .map(|row: Vec<bool>| row[i])
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::{
        evaluate_access_matrix, rotate_matrix, AllowList, RedisCache, Requirement, Role, User,
    };
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

    #[test]
    fn rotate_matrix_test() {
        let original = vec![
            vec![true, true, true, false, false],
            vec![true, true, true, true, true],
            vec![true, false, true, true, true],
            vec![true, true, true, false, true],
            vec![true, true, true, false, true],
        ];
        let rotated = vec![
            vec![true, true, true, true, true],
            vec![true, true, false, true, true],
            vec![true, true, true, true, true],
            vec![false, true, true, false, false],
            vec![false, true, true, true, true],
        ];

        assert_eq!(rotate_matrix(&original, 5), rotated);
    }

    #[test]
    fn evaluate_access_matrix_test() {
        let access_matrix = vec![
            vec![true, true, true, true, true],
            vec![true, true, false, true, true],
            vec![true, true, true, true, true],
            vec![false, true, true, false, false],
            vec![false, true, true, true, true],
        ];

        let logic = "(0 AND 1) OR (2 OR 3) AND 4";

        assert_eq!(
            evaluate_access_matrix(&access_matrix, logic).unwrap(),
            vec![true, true, true, false, true]
        );
    }

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
            metadata: serde_json::to_string(&token_type).unwrap(),
            relation,
        };

        let role = Role {
            id: "420".to_string(),
            logic: "0".to_string(),
            filter: Some(allowlist),
            requirements: vec![req],
        };

        let mut redis_cache = RedisCache::default();

        let client = reqwest::Client::new();

        assert_eq!(
            role.check_batch(&mut redis_cache, &client, &users)
                .await
                .unwrap(),
            &[true, true, false]
        );
    }
}
