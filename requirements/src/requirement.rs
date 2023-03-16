use super::{
    request::*,
    utils::{hash_string_to_scalar, parse_result},
};
use futures::future::join_all;
use guild_common::{Relation, Scalar, TokenType};
use guild_providers::{evm::Provider, BalanceQuerier, BalancyError, RpcError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Requirement {
    pub type_id: String,
    pub request: Request,
    pub identity_id: String,
    pub relation: Relation,
}

#[derive(Error, Debug)]
pub enum RequirementError {
    #[error("{0}")]
    ConversionFailed(String),
    #[error(transparent)]
    RpcError(#[from] RpcError),
    #[error(transparent)]
    BalancyError(#[from] BalancyError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

impl Requirement {
    pub async fn check(
        &self,
        client: &reqwest::Client,
        identity: &str,
    ) -> Result<bool, RequirementError> {
        let Request {
            base_url,
            method,
            data,
            auth,
            path,
        } = &self.request;

        if &self.type_id == "evm_balance" {
            if let Data::JsonBody(body) = data {
                let token_type = match body["type"].as_str().unwrap_or("") {
                    "fungible" => TokenType::Fungible {
                        address: body["address"].as_str().unwrap_or("").to_string(),
                    },
                    "non_fungible" => TokenType::NonFungible {
                        address: body["address"].as_str().unwrap_or("").to_string(),
                        id: match &body["id"] {
                            Value::String(id) => Some(id.clone()),
                            _ => None,
                        },
                    },
                    "special" => TokenType::Special {
                        address: body["address"].as_str().unwrap_or("").to_string(),
                        id: match &body["id"] {
                            Value::String(id) => Some(id.clone()),
                            _ => None,
                        },
                    },
                    _ => TokenType::Native,
                };

                let balance = Provider
                    .get_balance(client, base_url, &token_type, identity)
                    .await?;

                return Ok(self.relation.assert(&balance));
            } else {
                return Err(RequirementError::ConversionFailed(
                    "Wrong data type".to_string(),
                ));
            }
        }

        let url = if let Data::UrlEncoded(param) = data {
            format!("{base_url}?{param}={identity}")
        } else {
            base_url.to_string()
        };

        let mut builder = match method {
            Method::Get => client.get(url),
            Method::Post => client.post(url),
            _ => client.get(url),
        };

        if let Auth::Bearer(token) = auth {
            builder = builder.bearer_auth(token);
        }

        if let Data::JsonBody(body) = data {
            builder = builder.json(&body);
        }

        let result: Value = builder.send().await?.json().await?;
        let parsed = parse_result(result, path);

        Ok(check_access(parsed, &self.relation))
    }

    pub async fn check_batch(
        &self,
        client: &reqwest::Client,
        identities: &[String],
    ) -> Result<Vec<bool>, RequirementError> {
        join_all(
            identities
                .iter()
                .map(|identity| self.check(client, identity)),
        )
        .await
        .into_iter()
        .collect()
    }
}

fn check_access(value: Value, relation: &Relation) -> bool {
    match value {
        Value::Array(array) => array
            .iter()
            .any(|item| relation.assert(&hash_string_to_scalar(item.as_str().unwrap_or_default()))),
        Value::Bool(bool) => relation.assert(&Scalar::from(bool)),
        Value::Number(number) => relation.assert(&number.as_f64().unwrap_or_default()),
        Value::String(string) => relation.assert(&hash_string_to_scalar(&string)),
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use crate::requirement::{check_access, hash_string_to_scalar, Relation};
    #[cfg(feature = "check")]
    use crate::{balance::Balance, requirement::Requirement};
    #[cfg(feature = "check")]
    use guild_common::{Chain, TokenType};
    use serde_json::json;

    #[test]
    fn check_access_test() {
        let body_1 = json!("batman");
        let body_2 = json!(["superman", "batman", "aquaman"]);
        let body_3 = json!(true);
        let body_4 = json!(69);

        let relation_1_2 = Relation::EqualTo(hash_string_to_scalar("batman"));
        let relation_3 = Relation::EqualTo(1.0);
        let relation_4 = Relation::EqualTo(69.0);

        assert!(check_access(body_1, &relation_1_2));
        assert!(check_access(body_2, &relation_1_2));
        assert!(check_access(body_3, &relation_3));
        assert!(check_access(body_4, &relation_4));
    }

    #[tokio::test]
    #[cfg(feature = "check")]
    async fn requirement_check_test() {
        let balance_check = Balance {
            chain: Chain::Ethereum,
            token_type: TokenType::NonFungible {
                address: "0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85".to_string(),
                id: None,
            },
            relation: Relation::GreaterThan(0.0),
        };

        let req = Requirement::from(balance_check);
        let client = reqwest::Client::new();

        assert!(req
            .check(&client, "0xe43878ce78934fe8007748ff481f03b8ee3b97de")
            .await
            .unwrap());
    }
}
