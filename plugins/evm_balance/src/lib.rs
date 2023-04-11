#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![allow(clippy::multiple_crate_versions)]
#![deny(unused_crate_dependencies)]

mod balance;

use balance::EvmProvider;
use guild_common::{Relation, TokenType, User};
use reqwest::Client;
use tokio::runtime::Runtime;

#[no_mangle]
pub fn check(
    client: &'static Client,
    users: &[User],
    metadata: &str,
    secrets: &str,
) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
    let provider: EvmProvider = serde_json::from_str(secrets)?;
    let (token_type, relation): (TokenType, Relation<f64>) = serde_json::from_str(metadata)?;

    let addresses_with_ids: Vec<(u64, &str)> = users
        .iter()
        .flat_map(|user| {
            user.identities("evm_address")
                .map(|identities| {
                    identities
                        .iter()
                        .map(|address| (user.id, address.as_ref()))
                        .collect::<Vec<(u64, &str)>>()
                })
                .unwrap_or_default()
        })
        .collect();

    let addresses: Vec<&str> = addresses_with_ids
        .iter()
        .map(|(_, address)| *address)
        .collect();

    let rt = Runtime::new()?;

    let accesses: Vec<_> = rt.block_on(async {
        provider
            .get_balance_batch(client, token_type, &addresses)
            .await
            .map(|res| res.iter().map(|b| relation.assert(b)).collect())
    })?;

    let id_accesses = addresses_with_ids
        .iter()
        .zip(accesses.iter())
        .map(|((user_id, _), access)| (*user_id, *access))
        .collect::<Vec<(u64, bool)>>();

    let res = users
        .iter()
        .map(|user| {
            id_accesses
                .iter()
                .filter_map(|(i, access)| if &user.id == i { Some(*access) } else { None })
                .any(|b| b)
        })
        .collect();

    Ok(res)
}
