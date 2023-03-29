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
) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
    let provider: EvmProvider = serde_json::from_str(secrets)?;
    let (token_type, relation): (TokenType, Relation<f64>) = serde_json::from_str(metadata)?;

    let addresses: Vec<String> = users
        .iter()
        .flat_map(|user| user.identities("evm_address").cloned().unwrap_or_default())
        .collect();

    let rt = runtime::Runtime::new()?;

    Ok(rt.block_on(async {
        provider
            .get_balance_batch(client, token_type, &addresses)
            .await
            .map(|res| res.iter().map(|b| relation.assert(b)).collect())
    })?)
}
