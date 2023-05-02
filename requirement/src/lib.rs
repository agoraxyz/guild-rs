#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

pub mod relation;
pub mod token;

use guild_common::Scalar;
use serde::{Deserialize, Serialize};
pub use serde_cbor::{from_slice as cbor_deserialize, to_vec as cbor_serialize};

pub type Prefix = [u8; 8];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Requirement {
    pub prefix: Prefix,
    pub metadata: Vec<u8>,
    pub relation: relation::Relation<Scalar>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequirementsWithLogic {
    pub requirements: Vec<Requirement>,
    pub logic: String,
}

#[derive(Debug, Clone)]
pub struct SerializedRequirementsWithLogic {
    pub requirements: Vec<Vec<u8>>,
    pub logic: Vec<u8>,
}

impl TryFrom<RequirementsWithLogic> for SerializedRequirementsWithLogic {
    type Error = serde_cbor::Error;
    fn try_from(value: RequirementsWithLogic) -> Result<Self, Self::Error> {
        let requirements = value
            .requirements
            .into_iter()
            .map(|x| cbor_serialize(&x))
            .collect::<Result<Vec<_>, _>>()?;
        let logic = cbor_serialize(&value.logic)?;
        Ok(Self {
            requirements,
            logic,
        })
    }
}

impl TryFrom<SerializedRequirementsWithLogic> for RequirementsWithLogic {
    type Error = serde_cbor::Error;
    fn try_from(value: SerializedRequirementsWithLogic) -> Result<Self, Self::Error> {
        let requirements = value
            .requirements
            .into_iter()
            .map(|x| cbor_deserialize(&x))
            .collect::<Result<Vec<_>, _>>()?;
        let logic = cbor_deserialize(&value.logic)?;
        Ok(Self {
            requirements,
            logic,
        })
    }
}

/*
impl Requirement {
    pub fn check(
        &self,
        client: &Client,
        plugin_path: &str,
        secrets: &str,
    ) -> Result<Scalar, Error> {
        let lib = unsafe { Library::new(plugin_path) }?;
        let retrieve: Symbol<extern "C" fn(&Client, &[User], &str, &str) -> Result<Data, Error>> =
            unsafe { lib.get(b"retrieve") }?;

        let data = retrieve(client, users, &self.metadata, &secrets.to_string())?;

        let res = data
            .iter()
            .map(|values| values.iter().any(|v| self.relation.assert(v)))
            .collect();

        Ok(res)
    }
}
*/

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
