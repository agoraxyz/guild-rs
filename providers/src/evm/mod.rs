mod balancy;
mod jsonrpc;

pub use balancy::BalancyError;
#[cfg(feature = "balancy")]
pub use balancy::BalancyProvider as Provider;
use config::{Config, File};
#[cfg(not(feature = "balancy"))]
pub use jsonrpc::RpcProvider as Provider;
pub use jsonrpc::{get_erc20_decimals, RpcError};
use redis::{Commands, Connection, RedisError};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use thiserror::Error;

#[cfg(not(any(test, feature = "nomock")))]
const CONFIG_PATH: &str = "providers.json";
#[cfg(any(test, feature = "nomock"))]
const CONFIG_PATH: &str = "../providers.json";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EvmProvider {
    pub rpc_url: String,
    pub contract: String,
    pub balancy_id: Option<u8>,
}

#[derive(Error, Debug)]
pub enum ProviderConfigError {
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),
    #[error("Chain `{0}` is not supported")]
    ChainNotSupported(String),
    #[error("Field `{0}` has not been set")]
    FieldNotSet(String),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

fn get_redis_connection() -> Result<Connection, RedisError> {
    redis::Client::open("redis://127.0.0.1/")?.get_connection()
}

fn get_provider(chain: &str) -> Result<EvmProvider, ProviderConfigError> {
    let mut con = get_redis_connection().ok();

    if let Some(con) = con.as_mut() {
        if let Ok(entry) = con.get::<&str, String>(chain) {
            if let Ok(provider) = serde_json::from_str(&entry) {
                return Ok(provider);
            } else {
                let _: Result<(), _> = con.del(chain);
            }
        }
    };

    let settings = Config::builder()
        .add_source(File::from(Path::new(CONFIG_PATH)))
        .build()?;

    let map = settings.try_deserialize::<HashMap<String, EvmProvider>>()?;

    if let Some(provider) = map.get(chain).cloned() {
        if let Some(con) = con.as_mut() {
            let _: Result<(), _> = con.set(chain, serde_json::to_string(&provider).unwrap());
        }

        Ok(provider)
    } else {
        Err(ProviderConfigError::FieldNotSet(chain.to_string()))
    }
}

#[cfg(all(test, feature = "nomock"))]
mod common {
    pub const USER_1_ADDR: &str = "0xe43878ce78934fe8007748ff481f03b8ee3b97de";
    pub const USER_2_ADDR: &str = "0x14ddfe8ea7ffc338015627d160ccaf99e8f16dd3";
    pub const USER_3_ADDR: &str = "0x283d678711daa088640c86a1ad3f12c00ec1252e";
    pub const ERC20_ADDR: &str = "0x458691c1692cd82facfb2c5127e36d63213448a8";
    pub const ERC721_ADDR: &str = "0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85";
    pub const ERC721_ID: &str =
        "61313325075603536901663283754390960556726744542208800735045237225934362163454";
    pub const ERC1155_ADDR: &str = "0x76be3b62873462d2142405439777e971754e8e77";
    pub const ERC1155_ID: usize = 10868;
}
