use super::{Requirement, Scalar};
use guild_plugin_manager::{Client, PluginManager};

impl Requirement {
    pub fn check(
        &self,
        client: Client,
        plugin_path: &PluginManager,
        secrets: &str,
    ) -> Result<Scalar, anyhow::Error> {
        todo!()
    }
}

/*
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

type Data = Vec<Vec<Scalar>>;
type Error = Box<dyn std::error::Error>;


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


#[cfg(test)]
mod test {
    use shiba as _;
    use super::{RedisCache, Requirement, User};
    use guild_common::{Chain, Relation, RequirementType, TokenType};
    use reqwest::Client;
    use tokio::runtime;

    const USERS: &str = r#"[
    {
        "id": 0,
        "identities": {
            "evm_address": ["0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"],
            "sol_pubkey": ["5MLhcU2vPXHwxUFXQJXYGQcFfetTthDajWf4CgSYtMK9"]
        }
    },
    {
        "id": 1,
        "identities": {
            "evm_address": ["0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3"],
            "sol_pubkey": ["4fYNw3dojWmQ4dXtSGE9epjRGy9pFSx62YypT7avPYvA"]
        }
    },
    {
        "id": 2,
        "identities": {
            "evm_address": ["0x283d678711daa088640c86a1ad3f12c00ec1252e"],
            "sol_pubkey": ["vines1vzrYbzLMRdu58ou5XTby4qAqVRLmqo36NKPTg"]
        }
    }
    ]"#;

    #[test]
    fn requirement_check() {
        let token_type = TokenType::Fungible {
            address: "0x458691c1692cd82facfb2c5127e36d63213448a8".to_string(),
        };

        let relation_1 = Relation::GreaterThan(0.0);

        let evm_balance = Requirement {
            id: "69".to_string(),
            typ: RequirementType::EvmBalance.to_string(),
            config_key: Chain::Ethereum.to_string(),
            metadata: serde_json::to_string(&token_type).unwrap(),
            relation: relation_1,
        };

        let relation_2 = Relation::GreaterThan(420.0);

        let sol_balance = Requirement {
            id: "99".to_string(),
            typ: RequirementType::SolBalance.to_string(),
            config_key: Chain::SolanaMain.to_string(),
            metadata: String::new(),
            relation: relation_2,
        };

        let mut redis_cache = RedisCache::default();
        let client = Client::new();
        let users: Vec<User> = serde_json::from_str(USERS).unwrap();

        let rt = runtime::Runtime::new().unwrap();

        rt.block_on(async {
            assert_eq!(
                evm_balance
                    .check(&mut redis_cache, &client, &users)
                    .unwrap(),
                vec![false, true, false]
            );

            assert_eq!(
                sol_balance
                    .check(&mut redis_cache, &client, &users)
                    .unwrap(),
                vec![true, true, false]
            );
        });
    }
}
*/
