#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

pub use allowlist::AllowList;
use guild_common::User;
use guild_requirement::Requirement;
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
    Requiem(#[from] requiem::ParseError),
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
        let accesses: Vec<_> = self
            .requirements
            .iter()
            .map(|req| req.check(client, users))
            .collect();

        dbg!(accesses);

        todo!()
    }
}

#[cfg(test)]
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
                "0xe43878ce78934fe8007748ff481f03b8ee3b97de".to_string(),
                "0x14ddfe8ea7ffc338015627d160ccaf99e8f16dd3".to_string(),
            ],
        };

        let users: Vec<User> = serde_json::from_str(USERS).unwrap();

        let token_type = TokenType::NonFungible {
            address: "0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85".to_string(),
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
