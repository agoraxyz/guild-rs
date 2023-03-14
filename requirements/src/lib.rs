#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use allowlist::AllowList;
use guild_common::{Relation, Scalar};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use thiserror::Error;

mod allowlist;
mod balance;
mod user;

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
    JsonBody(String),
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
    pub id: String,
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
        //Ok(self.check_batch(client, &[identity]).await?[0])
    }

    pub async fn check_batch(
        &self,
        client: &reqwest::Client,
        identities: &[String],
    ) -> Result<Vec<bool>, RequirementError> {
        Ok(vec![true; identities.len()])
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
    let normalized_hash = (hash % prime) as f64 / prime as f64;

    normalized_hash
}

#[cfg(test)]
mod test {
    use super::{
        hash_string_to_f64, parse_result, user::User, Auth, Data, Method, Relation, Request,
        Requirement,
    };
    use serde_json::json;

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
    async fn requirement_check_test() {
        todo!();
    }
}
