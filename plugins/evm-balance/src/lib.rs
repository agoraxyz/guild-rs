#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![allow(clippy::multiple_crate_versions)]
#![deny(unused_crate_dependencies)]

mod balances;
mod call;
mod provider;

use guild_requirements::check::{CallOneInput, CallOneOutput};
use guild_requirements::{cbor_deserialize, token::TokenType};

#[no_mangle]
pub fn call_one(input: CallOneInput) -> Result<CallOneOutput, anyhow::Error> {
    let provider: provider::Provider = serde_json::from_str(input.serialized_secret)?;
    let token_type: TokenType = cbor_deserialize(&input.serialized_metadata)?;

    let balances: balances::Balances = futures::executor::block_on(async move {
        provider
            .balances(input.client.clone(), token_type, input.user)
            .await
    })?;

    Ok(balances.into_inner())
}
