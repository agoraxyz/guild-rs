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

#[cfg(test)]
mod test {
    use super::{
        json, Auth, Balance, Chain, Data, Method, Relation, Request, Requirement, TokenType,
    };

    #[test]
    fn requirement_from_balance() {
        let address = "0x458691c1692cd82facfb2c5127e36d63213448a8".to_string();
        let balance = Balance {
            chain: Chain::Ethereum,
            token_type: TokenType::Fungible { address },
            relation: Relation::EqualTo(69.420),
        };

        let request = Request {
            base_url: "ethereum".to_string(),
            method: Method::Get,
            data: Data::JsonBody(json!({
                "type": "fungible",
                "address": "0x458691c1692cd82facfb2c5127e36d63213448a8"
            })),
            auth: Auth::None,
            path: vec![],
        };

        let requirement = Requirement {
            type_id: "evm_balance".to_string(),
            request,
            identity_id: "evm_address".to_string(),
            relation: Relation::EqualTo(69.420),
        };

        assert_eq!(Requirement::from(balance), requirement);
    }
}
