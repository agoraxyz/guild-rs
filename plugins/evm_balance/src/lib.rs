#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![allow(clippy::multiple_crate_versions)]
#![deny(unused_crate_dependencies)]

mod balance;

use balance::EvmProvider;
use guild_common::{Scalar, TokenType, User};
use reqwest::Client;
use tokio::runtime::Runtime;

#[no_mangle]
pub fn retrieve(
    client: &'static Client,
    users: &[User],
    metadata: &str,
    secrets: &str,
) -> Result<Vec<Vec<Scalar>>, Box<dyn std::error::Error>> {
    let provider: EvmProvider = serde_json::from_str(secrets)?;
    let token_type: TokenType = serde_json::from_str(metadata)?;

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

    let balances: Vec<_> = rt.block_on(async {
        provider
            .get_balance_batch(client, token_type, &addresses)
            .await
    })?;

    let id_balances = addresses_with_ids
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
