use ethereum_types::U256;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RpcResponse {
    pub result: U256,
}
