mod call;
mod calldata;
mod multicall;

use call::Call;
use calldata::CallData;
use multicall::Multicall;

use crate::balances::Balances;
use guild_common::Scalar;
use reqwest::blocking::Client;

use std::str::FromStr;

const ETH_BALANCE_NORMALIZER: Scalar = 10_u128.pow(18) as Scalar;

pub fn eth_balances(
    client: Client,
    addresses: &[String],
    multicall_contract: String,
    rpc_url: &str,
) -> Result<Balances, anyhow::Error> {
    let multicall = Multicall::eth_balances(addresses);
    let call = multicall.aggregate(multicall_contract.clone(), multicall_contract);
    let response = call.dispatch(client, rpc_url)?;
    let mut balances = Balances::from_response(&response)?;
    balances.normalize(ETH_BALANCE_NORMALIZER);
    Ok(balances)
}

pub fn erc20_balances(
    client: Client,
    addresses: &[String],
    multicall_contract: String,
    contract: String,
    rpc_url: &str,
) -> Result<Balances, anyhow::Error> {
    let multicall = Multicall::erc20_balances(addresses);
    let call = multicall.aggregate(multicall_contract, contract.clone());
    let response = call.dispatch(client.clone(), rpc_url)?;
    let mut balances = Balances::from_response(&response)?;
    let decimals_call = Call::new(contract, CallData::erc20_decimals());
    let response = decimals_call.dispatch(client, rpc_url)?;
    let decimals = convert_decimals(&response)?;
    balances.normalize(10u128.pow(decimals) as Scalar);
    Ok(balances)
}

pub fn erc721_balances(
    client: Client,
    addresses: &[String],
    multicall_contract: String,
    contract: String,
    rpc_url: &str,
) -> Result<Balances, anyhow::Error> {
    let multicall = Multicall::erc721_balances(addresses);
    let call = multicall.aggregate(multicall_contract, contract);
    let response = call.dispatch(client, rpc_url)?;
    Balances::from_response(&response)
}

pub fn erc721_ownership(
    client: Client,
    addresses: &[String],
    contract: String,
    id: String,
    rpc_url: &str,
) -> Result<Balances, anyhow::Error> {
    let hex_id = dec_to_hex(&id)?;
    let call = Call::new(contract, CallData::erc721_owner(&hex_id));
    let response = call.dispatch(client, rpc_url)?;
    let trimmed = response
        .trim_start_matches("0x")
        .chars()
        .skip_while(|&c| c == '0')
        .collect::<String>()
        .to_lowercase();
    Ok(Balances::new(
        addresses
            .iter()
            .map(|address| address.trim_start_matches("0x").to_lowercase() == trimmed)
            .map(Scalar::from)
            .collect(),
    ))
}

pub fn erc1155_balances(
    client: Client,
    addresses: &[String],
    contract: String,
    id: String,
    rpc_url: &str,
) -> Result<Balances, anyhow::Error> {
    let hex_id = dec_to_hex(&id)?;
    let call = Call::new(contract, CallData::erc1155_balance_batch(addresses, &hex_id));
    let response = call.dispatch(client, rpc_url)?;
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
