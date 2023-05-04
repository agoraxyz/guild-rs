#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![allow(clippy::multiple_crate_versions)]
#![deny(unused_crate_dependencies)]

use guild_requirements::check::CallOneInput;

#[no_mangle]
pub fn call_one(input: CallOneInput) -> Result<String, anyhow::Error> {
    let provider: serde_json::Value = serde_json::from_str(input.serialized_secret)?;

    Ok(provider["RPC_URL"].to_string())
}
