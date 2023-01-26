pub mod balancy;

use ethereum_types::U256;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy)]
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

pub fn u256_from_str<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    U256::from_dec_str(s).map_err(D::Error::custom)
}
