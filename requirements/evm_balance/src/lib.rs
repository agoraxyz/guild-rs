#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![allow(clippy::multiple_crate_versions)]
#![deny(unused_crate_dependencies)]

mod providers;

use guild_common::{Relation, TokenType, User};
use providers::EvmProvider;
use reqwest::Client;
use tokio::runtime;

#[no_mangle]
pub fn check(
    client: &'static Client,
    users: &[User],
    metadata: &str,
    secrets: &str,
) -> Result<Vec<bool>, String> {
    let provider: EvmProvider = serde_json::from_str(secrets).unwrap();
    let (token_type, relation): (TokenType, Relation<f64>) =
        serde_json::from_str(metadata).unwrap();

    let addresses: Vec<String> = users
        .iter()
        .flat_map(|user| user.identities("evm_address").unwrap().clone())
        .collect();

    let rt = runtime::Runtime::new().unwrap();

    rt.block_on(async {
        match provider
            .get_balance_batch(client, token_type, &addresses)
            .await
        {
            Ok(res) => {
                Ok(res.iter().map(|balance| relation.assert(balance)).collect())
            }
            Err(err) => Err(err.to_string()),
        }
    })
}
