use crate::{
    evm::{
        jsonrpc::{create_payload, types::RpcResponse, JsonRpcMethods, RpcError, ETHEREUM},
        EvmChain,
    },
    CLIENT,
};
use ethereum_types::{Address, U256};

async fn call_contract(
    chain: EvmChain,
    contract_address: Address,
    data: String,
) -> Result<U256, RpcError> {
    let params = format!(
        "[
            {{
                \"to\": \"{contract_address:?}\",
                \"data\": \"{data}\"
            }},
            \"latest\"
        ]"
    );
    let payload = create_payload(JsonRpcMethods::EthCall, params, 1);

    let res: RpcResponse = CLIENT
        .read()
        .await
        .post(ETHEREUM)
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
    call_contract(chain, token_address, data).await
}

pub async fn get_erc721_balance(
    chain: EvmChain,
    token_address: Address,
    token_id: Option<U256>,
    user_address: Address,
) -> Result<U256, RpcError> {
    let addr = format!("{user_address:?}")[2..].to_string();
    let data = match token_id {
        Some(id) => format!("0x6352211e000000000000000000000000{addr}{id:064x}"),
        None => format!("0x70a08231000000000000000000000000{addr}"),
    };
    call_contract(chain, token_address, data).await
}

pub async fn get_erc1155_balance(
    chain: EvmChain,
    token_address: Address,
    token_id: Option<U256>,
    user_address: Address,
) -> Result<U256, RpcError> {
    let addr = format!("{user_address:?}")[2..].to_string();
    let data = match token_id {
        Some(id) => dbg!(format!("0x00fdd58e000000000000000000000000{addr}{id:064x}")),
        None => format!("0x70a08231000000000000000000000000{addr}"),
    };
    call_contract(chain, token_address, data).await
}
