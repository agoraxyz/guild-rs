#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

pub use allowlist::AllowList;
#[cfg(any(feature = "frontend", feature = "test"))]
pub use balance::Balance;
pub use requirement::Requirement;
use serde_json::Value;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

mod allowlist;
mod balance;
mod request;
mod requirement;

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
