#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use config::{Config, File};
use guild_common::{Requirement, User};
use libloading::{Library, Symbol};
use reqwest::Client;
use serde_json::Value;
use std::{collections::HashMap, path::Path};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error("Value not found for key {0}")]
    NoSuchEntry(String),
}

#[cfg(not(test))]
const CONFIG_PATH: &str = "config.json";
#[cfg(test)]
const CONFIG_PATH: &str = "../config.json";

fn read_config(key: &str) -> Result<Value, ConfigError> {
    let settings = Config::builder()
        .add_source(File::from(Path::new(CONFIG_PATH)))
        .build()?;

    let map = settings.try_deserialize::<HashMap<String, Value>>()?;

    map.get(key)
        .cloned()
        .ok_or(ConfigError::NoSuchEntry(key.to_string()))
}

pub trait Checkable {
    fn check(&self, client: &Client, users: &[User]) -> Result<Vec<bool>, String>;
}

impl Checkable for Requirement {
    fn check(&self, client: &Client, users: &[User]) -> Result<Vec<bool>, String> {
        let path = read_config(&self.typ.to_string()).unwrap();
        let path_str = path.as_str().unwrap();

        let lib = unsafe { Library::new(path_str) }.unwrap();

        let check_req: Symbol<
            extern "C" fn(&Client, &[User], &str, &str) -> Result<Vec<bool>, String>,
        > = unsafe { lib.get(b"check") }.unwrap();

        let secrets = read_config(&self.config_key).unwrap();

        check_req(client, &users, &self.metadata, &secrets.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::{Checkable, Requirement, User};
    use guild_common::{Chain, Relation, RequirementType, TokenType};
    use reqwest::Client;
    use tokio::runtime;

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
    fn requirement_check() {
        let token_type = TokenType::Special {
            address: "0x76be3b62873462d2142405439777e971754e8e77".to_string(),
            id: None,
        };
        let relation = Relation::GreaterThan(0.0);

        let req = Requirement {
            id: "69".to_string(),
            typ: RequirementType::EvmBalance,
            config_key: Chain::Ethereum.to_string(),
            metadata: serde_json::to_string(&(token_type, relation)).unwrap(),
        };

        let client = Client::new();
        let users: Vec<User> = serde_json::from_str(USERS).unwrap();

        let rt = runtime::Runtime::new().unwrap();

        rt.block_on(async {
            assert_eq!(req.check(&client, &users), Ok(vec![false, false, true]));
        });
    }
}
