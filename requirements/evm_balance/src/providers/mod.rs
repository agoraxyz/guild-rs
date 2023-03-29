use guild_common::TokenType;
use serde::{Deserialize, Serialize};
#[cfg(feature = "nomock")]
use tokio as _;

mod balancy;
mod jsonrpc;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EvmProvider {
    pub rpc_url: String,
    pub contract: String,
    pub balancy_id: Option<u8>,
}

#[cfg(all(test, feature = "nomock"))]
mod common {
    pub const RPC_URL: &str = "https://eth.public-rpc.com";
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
