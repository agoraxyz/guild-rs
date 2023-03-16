#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

pub use allowlist::AllowList;
#[cfg(any(feature = "frontend", feature = "test"))]
pub use balance::Balance;
use futures::future::join_all;
use guild_common::{Relation, Scalar, TokenType};
use guild_providers::{evm::Provider, BalanceQuerier, BalancyError, RpcError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use thiserror::Error;

mod allowlist;
mod balance;

pub struct Role {
    pub id: String,
    pub filter: Option<AllowList<String>>,
    pub logic: String,
    pub requirements: Option<Vec<Requirement>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
    Patch,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Auth {
    None,
    ApiKey(String),
    Bearer(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Data {
    None,
    UrlEncoded(String),
    JsonBody(Value),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Request {
    pub base_url: String,
    pub method: Method,
    pub data: Data,
    pub auth: Auth,
    pub path: Vec<Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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

        if &self.type_id == "evmbalance" {
            if let Data::JsonBody(body) = data {
                let token_type = match body["type"].as_str().unwrap_or("") {
                    "fungible" => TokenType::Fungible {
                        address: body["address"].as_str().unwrap_or("").to_string(),
                    },
                    "non_fungible" => TokenType::NonFungible {
                        address: body["address"].as_str().unwrap_or("").to_string(),
                        id: match body["id"].as_str().unwrap_or("") {
                            "" => None,
                            id => Some(id.to_string()),
                        },
                    },
                    "special" => TokenType::Special {
                        address: body["address"].as_str().unwrap_or("").to_string(),
                        id: match body["id"].as_str().unwrap_or("") {
                            "" => None,
                            id => Some(id.to_string()),
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

        let access = match parsed {
            Value::Array(array) => match self.relation {
                Relation::EqualTo(value) => array
                    .iter()
                    .any(|item| hash_string_to_f64(&item.to_string()) == value),
                _ => true,
            },
            Value::Bool(bool) => self.relation.assert(&Scalar::from(bool)),
            Value::Number(number) => self.relation.assert(&number.as_f64().unwrap_or_default()),
            Value::String(string) => self.relation.assert(&hash_string_to_f64(&string)),
            _ => false,
        };

        Ok(access)
    }

    pub async fn check_batch(
        &self,
        client: &reqwest::Client,
        identities: &[String],
    ) -> Result<Vec<bool>, RequirementError> {
        join_all(
            identities
                .iter()
                .map(|identity| async { self.check(client, identity).await }),
        )
        .await
        .into_iter()
        .collect()
    }
}

fn parse_result(result: Value, path: &[Value]) -> Value {
    path.iter()
        .fold(&result, |current_value, field| match field {
            Value::String(k) => &current_value[k.as_str()],
            Value::Number(i) => &current_value[i.as_u64().unwrap_or_default() as usize],
            _ => panic!("Invalid path element"),
        })
        .to_owned()
}

fn hash_string_to_f64(s: &str) -> f64 {
    let mut hasher = DefaultHasher::new();

    s.hash(&mut hasher);

    let hash = hasher.finish() as u128;
    let prime = 18446744073709551629_u128; // Mersenne prime M61

    (hash % prime) as f64 / prime as f64
}

#[cfg(test)]
mod test {
    use super::{hash_string_to_f64, parse_result};
    #[cfg(feature = "test")]
    use super::{Balance, Relation, Requirement};
    #[cfg(feature = "test")]
    use guild_common::{Chain, TokenType};
    use serde_json::json;

    use tokio as _;

    #[test]
    fn parse_result_test() {
        let result = json!({
            "users": [
                { "name": "Walter", "balance": 99.4 },
                { "name": "Jesse", "balance": 420.0 },
                { "name": "Jimmy", "balance": 69.0 },
            ]
        });
        let path = [json!("users"), json!(1), json!("balance")];
        let balance = parse_result(result, &path);

        assert_eq!(balance.to_string().parse::<f64>().unwrap(), 420.0);
    }

    #[test]
    fn hash_string_to_f64_test() {
        assert_eq!(
            hash_string_to_f64("Lorem ipsum dolor sit amet"),
            0.7593360189081984
        );
    }

    #[tokio::test]
    #[cfg(feature = "test")]
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
