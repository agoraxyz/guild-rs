mod balancy;
mod jsonrpc;

pub use balancy::BalancyError;
#[cfg(feature = "balancy")]
pub use balancy::BalancyProvider as Provider;
use config::{Config, File};
#[cfg(not(feature = "balancy"))]
pub use jsonrpc::RpcProvider as Provider;
pub use jsonrpc::{get_erc20_decimals, RpcError};
use primitive_types::H160 as Address;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use thiserror::Error;

#[cfg(not(any(test, feature = "nomock")))]
const CONFIG_PATH: &str = "providers.json";
#[cfg(any(test, feature = "nomock"))]
const CONFIG_PATH: &str = "../providers.json";

#[derive(Clone, Deserialize)]
struct EvmProvider {
    pub rpc_url: String,
    pub contract: Address,
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
}

fn get_provider(chain: &str) -> Result<EvmProvider, ProviderConfigError> {
    let settings = Config::builder()
        .add_source(File::from(Path::new(CONFIG_PATH)))
        .build()?;

    let map = settings.try_deserialize::<HashMap<String, EvmProvider>>()?;

    map.get(chain)
        .ok_or(ProviderConfigError::FieldNotSet(chain.to_string()))
        .cloned()
}

#[cfg(all(test, feature = "nomock"))]
mod common {
    pub const USER_1_ADDR: &str = "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE";
    pub const USER_2_ADDR: &str = "0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3";
    pub const USER_3_ADDR: &str = "0x283d678711daa088640c86a1ad3f12c00ec1252e";
    pub const ERC20_ADDR: &str = "0x458691c1692cd82facfb2c5127e36d63213448a8";
    pub const ERC721_ADDR: &str = "0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85";
    pub const ERC721_ID: &str =
        "61313325075603536901663283754390960556726744542208800735045237225934362163454";
    pub const ERC1155_ADDR: &str = "0x76be3b62873462d2142405439777e971754e8e77";
    pub const ERC1155_ID: usize = 10868;
}
