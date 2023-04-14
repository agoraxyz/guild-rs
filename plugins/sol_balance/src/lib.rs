#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![allow(clippy::multiple_crate_versions)]
#![deny(unused_crate_dependencies)]

use guild_common::{Scalar, User};
use reqwest::Client;
use serde_json::{json, Value};
use thiserror::Error;
use tokio::runtime::Runtime;

#[derive(Error, Debug)]
pub enum SolanaError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Other(String),
}

fn create_payload(method: &str, params: Value, id: u32) -> Value {
    json!({
        "method"  : method,
        "params"  : params,
        "id"      : id,
        "jsonrpc" : "2.0"
    })
}

async fn get_balance_batch(
    client: &Client,
    base_url: &str,
    pubkeys: &[&str],
) -> Result<Vec<Scalar>, SolanaError> {
    let params = json!([
        pubkeys,
        {
            "encoding": "jsonParsed"
        }
    ]);
    let payload = create_payload("getMultipleAccounts", params, 1);

    let res: Value = client
        .post(base_url)
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    match res["result"]["value"].as_array() {
        Some(values) => Ok(values
            .iter()
            .map(|value| value["lamports"].as_f64().unwrap())
            .collect()),
        None => Err(SolanaError::Other(
            "Failed to deserialize result".to_string(),
        )),
    }
}

#[no_mangle]
pub fn retrieve(
    client: &'static Client,
    users: &[User],
    _metadata: &str,
    secrets: &str,
) -> Result<Vec<Vec<Scalar>>, Box<dyn std::error::Error>> {
    let secret: Value = serde_json::from_str(secrets)?;
    let base_url = secret.as_str().unwrap_or_default();

    let pubkeys_with_ids: Vec<(u64, &str)> = users
        .iter()
        .flat_map(|user| {
            user.identities("sol_pubkey")
                .map(|identities| {
                    identities
                        .iter()
                        .map(|pubkey| (user.id, pubkey.as_ref()))
                        .collect::<Vec<(u64, &str)>>()
                })
                .unwrap_or_default()
        })
        .collect();

    let pubkeys: Vec<&str> = pubkeys_with_ids.iter().map(|(_, pubkey)| *pubkey).collect();

    let rt = Runtime::new()?;

    let balances: Vec<_> = rt.block_on(get_balance_batch(client, base_url, &pubkeys))?;

    let id_balances = pubkeys_with_ids
        .iter()
        .zip(balances.iter())
        .map(|((user_id, _), balance)| (*user_id, *balance))
        .collect::<Vec<(u64, Scalar)>>();

    let res = users
        .iter()
        .map(|user| {
            id_balances
                .iter()
                .filter_map(|(i, balance)| if &user.id == i { Some(*balance) } else { None })
                .collect()
        })
        .collect();

    Ok(res)
}

#[cfg(test)]
mod test {
    use super::get_balance_batch;

    const BASE_URL: &str = "https://api.mainnet-beta.solana.com";

    #[tokio::test]
    async fn test_balance_batch() {
        let client = reqwest::Client::new();

        let pubkeys = &[
            "5MLhcU2vPXHwxUFXQJXYGQcFfetTthDajWf4CgSYtMK9",
            "4fYNw3dojWmQ4dXtSGE9epjRGy9pFSx62YypT7avPYvA",
        ];

        let res = get_balance_batch(&client, BASE_URL, pubkeys).await.unwrap();

        assert_eq!(res, [1761523130.0, 2000000.0]);
    }
}
