#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![allow(clippy::multiple_crate_versions)]
#![deny(unused_crate_dependencies)]

mod balances;
mod call;
mod provider;

use guild_plugin_manager::{CallOneInput, CallOneResult};
use guild_requirement::{cbor_deserialize, token::TokenType};

#[no_mangle]
pub fn call_one(input: CallOneInput) -> CallOneResult {
    // TODO error handling
    let provider: provider::Provider = cbor_deserialize(&input.serialized_secrets).unwrap();
    let token_type: TokenType = cbor_deserialize(&input.serialized_metadata).unwrap();

    let balances: balances::Balances = futures::executor::block_on(async move {
        provider
            .balances(input.client.clone(), token_type, input.user)
            .await
    })
    .unwrap();

    Ok(balances.into_inner())
}

// TODO call_batch
