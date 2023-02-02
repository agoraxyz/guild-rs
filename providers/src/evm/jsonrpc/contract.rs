use crate::{
    evm::{
        jsonrpc::{create_payload, RpcError, RpcResponse, PROVIDERS},
        EvmChain,
    },
    CLIENT,
};
use ethereum_types::{Address, U256};
use rusty_gate_common::address;
use std::str::FromStr;

async fn call_contract(
    chain: EvmChain,
    contract_address: Address,
    data: String,
) -> Result<String, RpcError> {
    let Some(provider) = PROVIDERS.get(&chain) else {
       return Err(RpcError::ChainNotSupported(format!("{chain:?}")));
    };

    let params = format!(
        "[
            {{
                \"to\"   : \"{contract_address:?}\",
                \"data\" : \"{data}\"
            }},
            \"latest\"
        ]"
    );
    let payload = create_payload("eth_call", params, 1);

    let res: RpcResponse = CLIENT
        .write()
        .await
        .post(&provider.rpc_url)
        .body(payload)
        .send()
        .await?
        .json()
        .await?;

    Ok(res.result)
}

pub async fn get_erc20_balance(
    chain: EvmChain,
    token_address: Address,
    user_address: Address,
) -> Result<U256, RpcError> {
    let addr = format!("{user_address:?}")[2..].to_string();
    let data = format!("0x70a08231000000000000000000000000{addr}");
    let balance = call_contract(chain, token_address, data).await?;

    U256::from_str(&balance).map_err(|err| RpcError::Other(err.to_string()))
}

pub async fn get_erc721_balance(
    chain: EvmChain,
    token_address: Address,
    token_id: Option<U256>,
    user_address: Address,
) -> Result<U256, RpcError> {
    let addr = format!("{user_address:?}")[2..].to_string();
    match token_id {
        Some(id) => {
            let data = format!("0x6352211e{id:064x}");
            let addr = call_contract(chain, token_address, data).await?;

            Ok(U256::from((address!(&addr[26..]) == user_address) as u8))
        }
        None => {
            let data = format!("0x70a08231000000000000000000000000{addr}");
            let balance = call_contract(chain, token_address, data).await?;

            U256::from_str(&balance).map_err(|err| RpcError::Other(err.to_string()))
        }
    }
}

pub async fn get_erc1155_balance(
    chain: EvmChain,
    token_address: Address,
    token_id: U256,
    user_address: Address,
) -> Result<U256, RpcError> {
    let addr = format!("{user_address:?}")[2..].to_string();
    let data = format!("0x00fdd58e000000000000000000000000{addr}{token_id:064x}");
    let balance = call_contract(chain, token_address, data).await?;

    U256::from_str(&balance).map_err(|err| RpcError::Other(err.to_string()))
}
