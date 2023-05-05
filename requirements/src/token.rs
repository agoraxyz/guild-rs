#[cfg(not(feature = "std"))]
use super::String;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum TokenType {
    Native,
    Fungible { address: String },
    NonFungible { address: String, id: Option<String> },
    Special { address: String, id: Option<String> },
}
