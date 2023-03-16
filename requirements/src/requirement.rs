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
                    .any(|item| hash_string_to_scalar(&item.to_string()) == value),
                _ => true,
            },
            Value::Bool(bool) => self.relation.assert(&Scalar::from(bool)),
            Value::Number(number) => self.relation.assert(&number.as_f64().unwrap_or_default()),
            Value::String(string) => self.relation.assert(&hash_string_to_scalar(&string)),
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
                .map(|identity| self.check(client, identity)),
        )
        .await
        .into_iter()
        .collect()
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "test")]
    use super::{Relation, Requirement};
    #[cfg(feature = "test")]
    use crate::balance::Balance;
    #[cfg(feature = "test")]
    use guild_common::{Chain, TokenType};

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
