#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use config::{Config, File};
pub use db::RedisCache;
use guild_common::{Relation, Scalar, User};
use libloading::{Library, Symbol};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, path::Path};
use thiserror::Error;

mod db;

type Error = Box<dyn std::error::Error>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Requirement {
    pub id: String,
    pub typ: String,
    pub config_key: String,
    pub metadata: String,
    pub relation: Relation<Scalar>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error("Value not found for key {0}")]
    NoSuchEntry(String),
}

const CONFIG_PATH: &str = "config.json";

fn read_config(redis_cache: &mut RedisCache, key: &str) -> Result<Value, ConfigError> {
    if let Some(value) = redis_cache.read(key) {
        return Ok(value);
    }

    let config_path = std::env::var("CONFIG_PATH").unwrap_or(CONFIG_PATH.to_string());

    let settings = Config::builder()
        .add_source(File::from(Path::new(&config_path)))
        .build()?;

    let map = settings.try_deserialize::<HashMap<String, Value>>()?;

    if let Some(value) = map.get(key).cloned() {
        redis_cache.write(key, &value);

        Ok(value)
    } else {
        Err(ConfigError::NoSuchEntry(key.to_string()))
    }
}

impl Requirement {
    pub fn check(
        &self,
        redis_cache: &mut RedisCache,
        client: &Client,
        users: &[User],
    ) -> Result<Vec<bool>, Error> {
        let path = read_config(redis_cache, &self.typ.to_string())?;
        let path_str = path.as_str().unwrap_or_default();

        let lib = unsafe { Library::new(path_str) }?;

        let retrieve: Symbol<
            extern "C" fn(&Client, &[User], &str, &str) -> Result<Vec<bool>, Error>,
        > = unsafe { lib.get(b"retrieve") }?;

        let secrets = read_config(redis_cache, &self.config_key)?;

        retrieve(client, users, &self.metadata, &secrets.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::{RedisCache, Requirement, User};
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
        let token_type = TokenType::Fungible {
            address: "0x458691c1692cd82facfb2c5127e36d63213448a8".to_string(),
        };
        let relation = Relation::GreaterThan(0.0);

        let req = Requirement {
            id: "69".to_string(),
            typ: RequirementType::EvmBalance.to_string(),
            config_key: Chain::Ethereum.to_string(),
            metadata: serde_json::to_string(&token_type).unwrap(),
            relation,
        };

        let mut redis_cache = RedisCache::default();
        let client = Client::new();
        let users: Vec<User> = serde_json::from_str(USERS).unwrap();

        let rt = runtime::Runtime::new().unwrap();

        rt.block_on(async {
            assert_eq!(
                req.check(&mut redis_cache, &client, &users).unwrap(),
                vec![false, true, false]
            );
        });
    }
}
