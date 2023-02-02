mod balancy;
mod jsonrpc;

pub use balancy::BALANCY_PROVIDER;
use ethereum_types::{Address, U256};
pub use jsonrpc::RPC_PROVIDER;
use rusty_gate_common::address;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy, std::hash::Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvmChain {
    Ethereum,
    Polygon,
    Gnosis,
    Bsc,
    Fantom,
    Avalanche,
    Heco,
    Harmony,
    Goerli,
    Arbitrum,
    Celo,
    Optimism,
    Moonriver,
    Rinkeby,
    Metis,
    Cronos,
    Boba,
    Palm,
}

pub struct Provider {
    pub rpc_url: String,
    pub multicall_contract: Address,
}

macro_rules! dotenv {
    ($var: expr) => {
        match std::env::var($var) {
            Ok(val) => val,
            Err(_) => panic!("Environment variable `{}` not found", $var),
        }
    };
}

lazy_static::lazy_static! {
    pub static ref PROVIDERS: Arc<HashMap<EvmChain, Provider>> = Arc::new({
        dotenv::dotenv().ok();

        let mut providers = HashMap::new();

        providers.insert(
            EvmChain::Ethereum,
            Provider {
                rpc_url: dotenv!("ETHEREUM_RPC"),
                multicall_contract: address!("0x5ba1e12693dc8f9c48aad8770482f4739beed696"),
            }
        );
        providers.insert(
            EvmChain::Polygon,
            Provider {
                rpc_url: dotenv!("POLYGON_RPC"),
                multicall_contract: address!("0x11ce4B23bD875D7F5C6a31084f55fDe1e9A87507"),
            }
        );
        providers.insert(
            EvmChain::Bsc,
            Provider {
                rpc_url: dotenv!("BSC_RPC"),
                multicall_contract: address!("0x41263cba59eb80dc200f3e2544eda4ed6a90e76c")
            }
        );
        providers.insert(
            EvmChain::Gnosis,
            Provider {
                rpc_url: dotenv!("GNOSIS_RPC"),
                multicall_contract: address!("0xb5b692a88bdfc81ca69dcb1d924f59f0413a602a")
            }
        );
        providers.insert(
            EvmChain::Arbitrum,
            Provider {
                rpc_url: dotenv!("ARBITRUM_RPC"),
                multicall_contract: address!("0x52bfe8fE06c8197a8e3dCcE57cE012e13a7315EB")
            }
        );
        providers.insert(
            EvmChain::Goerli,
            Provider {
                rpc_url: dotenv!("GOERLI_RPC"),
                multicall_contract: address!("0x77dCa2C955b15e9dE4dbBCf1246B4B85b651e50e")
            }
        );

        providers
    });
}

pub fn u256_from_str<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    U256::from_dec_str(s).map_err(D::Error::custom)
}

#[cfg(test)]
mod common {
    pub const USER_1_ADDR: &str = "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE";
    pub const USER_2_ADDR: &str = "0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3";
    pub const USER_3_ADDR: &str = "0x283d678711daa088640c86a1ad3f12c00ec1252e";
    pub const ERC20_ADDR: &str = "0x458691c1692cd82facfb2c5127e36d63213448a8";
    pub const ERC721_ADDR: &str = "0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85";
    pub const ERC721_ID: &str =
        "61313325075603536901663283754390960556726744542208800735045237225934362163454";
    pub const ERC1155_ADDR: &str = "0x76be3b62873462d2142405439777e971754e8e77";
    pub const ERC1155_ID: &str = "10868";
}
