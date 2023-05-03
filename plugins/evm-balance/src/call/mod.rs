mod call;
mod calldata;
mod multicall;

use call::Call;
use calldata::CallData;
use multicall::Multicall;

use crate::balances::Balances;
use guild_common::Scalar;
use reqwest::Client;

use std::str::FromStr;

pub async fn eth_balances(
    client: Client,
    addresses: &[String],
    target: String,
    rpc_url: &str,
) -> Result<Balances, anyhow::Error> {
    let multicall = Multicall::eth_balances(addresses);
    let call = multicall.aggregate(target.clone(), target);
    let response = call.dispatch(client, rpc_url).await?;
    Balances::from_response(&response)
}

pub async fn erc20_balances(
    client: Client,
    addresses: &[String],
    target: String,
    contract: String,
    rpc_url: &str,
) -> Result<Balances, anyhow::Error> {
    let multicall = Multicall::erc20_balances(addresses);
    let call = multicall.aggregate(target, contract.clone());
    let response = call.dispatch(client.clone(), rpc_url).await?;
    let mut balances = Balances::from_response(&response)?;
    let decimals_call = Call::new(contract, CallData::erc20_decimals());
    let response = decimals_call.dispatch(client, rpc_url).await?;
    let decimals = convert_decimals(&response)?;
    balances.normalize(decimals);
    Ok(balances)
}

pub async fn erc721_balances(
    client: Client,
    addresses: &[String],
    target: String,
    contract: String,
    rpc_url: &str,
) -> Result<Balances, anyhow::Error> {
    let multicall = Multicall::erc721_balances(addresses);
    let call = multicall.aggregate(target, contract);
    let response = call.dispatch(client, rpc_url).await?;
    Balances::from_response(&response)
}

pub async fn erc721_ownership(
    client: Client,
    addresses: &[String],
    contract: String,
    id: String,
    rpc_url: &str,
) -> Result<Balances, anyhow::Error> {
    let hex_id = dec_to_hex(&id)?;
    let call = Call::new(contract, CallData::erc721_owner(&hex_id));
    let response = call.dispatch(client, rpc_url).await?;
    Ok(Balances::new(
        addresses
            .iter()
            .map(|address| address.to_lowercase() == response)
            .map(Scalar::from)
            .collect(),
    ))
}

pub async fn erc1155_balances(
    client: Client,
    addresses: &[String],
    contract: String,
    id: String,
    rpc_url: &str,
) -> Result<Balances, anyhow::Error> {
    let call = Call::new(contract, CallData::erc1155_balance_batch(addresses, &id));
    let response = call.dispatch(client, rpc_url).await?;
    Balances::from_special_response(&response)
}

fn dec_to_hex(input: &str) -> Result<String, anyhow::Error> {
    let parsed = primitive_types::U256::from_dec_str(input)?;
    Ok(format!("{:x}", parsed))
}

fn convert_decimals(input: &str) -> Result<u32, anyhow::Error> {
    let parsed = primitive_types::U256::from_str(input)?;
    Ok(parsed.as_u32())
}

#[test]
fn dec_to_hex_conversion() {
    assert_eq!(dec_to_hex("0").unwrap(), "0");
    assert_eq!(dec_to_hex("10").unwrap(), "a");
    assert_eq!(dec_to_hex("15").unwrap(), "f");
    assert_eq!(dec_to_hex("16").unwrap(), "10");
    assert_eq!(dec_to_hex("1024").unwrap(), "400");
    assert!(dec_to_hex("abc").is_err());
}
