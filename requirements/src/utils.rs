use guild_common::Scalar;
use serde_json::Value;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub fn parse_result(result: Value, path: &[Value]) -> Value {
    path.iter()
        .fold(&result, |current_value, field| match field {
            Value::String(k) => &current_value[k.as_str()],
            Value::Number(i) => &current_value[i.as_u64().unwrap_or_default() as usize],
            _ => panic!("Invalid path element"),
        })
        .to_owned()
}

const MERSENNE_PRIME: u128 = 18446744073709551629;

pub fn hash_string_to_scalar(s: &str) -> Scalar {
    let mut hasher = DefaultHasher::new();

    s.hash(&mut hasher);

    let hash = hasher.finish() as u128;

    (hash % MERSENNE_PRIME) as Scalar / MERSENNE_PRIME as Scalar
}

#[cfg(test)]
mod test {
    use super::{hash_string_to_scalar, parse_result, Scalar};
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

        assert_eq!(balance.to_string().parse::<Scalar>().unwrap(), 420.0);
    }

    #[test]
    fn hash_string_to_scalar_test() {
        assert_eq!(
            hash_string_to_scalar("Lorem ipsum dolor sit amet"),
            0.7593360189081984
        );
    }
}
