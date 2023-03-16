use super::{request::*, Requirement};
use guild_common::{Chain, Relation, RequirementType, TokenType};
use serde_json::json;

#[derive(Debug)]
pub struct Balance {
    pub chain: Chain,
    pub token_type: TokenType,
    pub relation: Relation,
}

impl From<Balance> for Requirement {
    fn from(value: Balance) -> Self {
        let data = match value.token_type {
            TokenType::Native => json!({
                "type": "native"
            }),
            TokenType::Fungible { address } => json!({
                "type": "fungible",
                "address": address
            }),
            TokenType::NonFungible { address, id } => json!({
                "type": "non_fungible",
                "address": address,
                "id": id
            }),
            TokenType::Special { address, id } => json!({
                "type": "special",
                "address": address,
                "id": id
            }),
        };

        let request = Request {
            base_url: value.chain.to_string(),
            method: Method::Get,
            data: Data::JsonBody(data),
            auth: Auth::None,
            path: vec![],
        };

        Self {
            type_id: RequirementType::EvmBalance.to_string(),
            request,
            identity_id: "evm_address".to_string(),
            relation: value.relation,
        }
    }
}
