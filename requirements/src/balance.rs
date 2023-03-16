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
        let (typ, address, id) = match value.token_type {
            TokenType::Native => ("native".to_string(), "".to_string(), "".to_string()),
            TokenType::Fungible { address } => ("fungible".to_string(), address, "".to_string()),
            TokenType::NonFungible { address, id } => {
                ("non_fungible".to_string(), address, id.unwrap_or_default())
            }
            TokenType::Special { address, id } => {
                ("special".to_string(), address, id.unwrap_or_default())
            }
        };

        let request = Request {
            base_url: value.chain.to_string(),
            method: Method::Get,
            data: Data::JsonBody(json!({
                "type": typ,
                "address": address,
                "id": id
            })),
            auth: Auth::None,
            path: vec![],
        };

        Self {
            type_id: format!("{:?}", RequirementType::EvmBalance).to_lowercase(),
            request,
            identity_id: "evmaddress".to_string(),
            relation: value.relation,
        }
    }
}
