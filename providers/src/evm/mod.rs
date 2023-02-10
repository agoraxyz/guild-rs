mod balancy;
mod jsonrpc;

#[cfg(feature = "balancy")]
pub use balancy::BalancyProvider as Provider;
#[cfg(not(feature = "balancy"))]
pub use jsonrpc::RpcProvider as Provider;
use serde::{Deserialize, Serialize};

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
    pub const ERC1155_ID: &str = "10868";
}
