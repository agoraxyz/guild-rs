use crate::evm::{
    jsonrpc::{create_payload, RpcError, RpcResponse, PROVIDERS},
    EvmChain,
};
use ethereum_types::{Address, U256};
use rusty_gate_common::address;
use std::str::FromStr;

async fn call_contract(
    client: &reqwest::Client,
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

    let res: RpcResponse = client
        .post(&provider.rpc_url)
        .body(payload)
        .send()
        .await?
        .json()
        .await?;

    Ok(res.result)
}

pub async fn get_erc20_decimals(
    client: &reqwest::Client,
    chain: EvmChain,
    token_address: Address,
) -> Result<U256, RpcError> {
    let data = "0x313ce567".to_string();
    let decimals = call_contract(client, chain, token_address, data).await?;

    U256::from_str(&decimals).map_err(|err| RpcError::Other(err.to_string()))
}

pub async fn get_erc20_balance(
    client: &reqwest::Client,
    chain: EvmChain,
    token_address: Address,
    user_address: Address,
) -> Result<U256, RpcError> {
    let addr = format!("{user_address:?}")[2..].to_string();
    let data = format!("0x70a08231000000000000000000000000{addr}");
    let balance = call_contract(client, chain, token_address, data).await?;

    U256::from_str(&balance).map_err(|err| RpcError::Other(err.to_string()))
}

pub async fn get_erc721_balance(
    client: &reqwest::Client,
    chain: EvmChain,
    token_address: Address,
    token_id: Option<U256>,
    user_address: Address,
) -> Result<U256, RpcError> {
    let addr = format!("{user_address:?}")[2..].to_string();
    match token_id {
        Some(id) => {
            let data = format!("0x6352211e{id:064x}");
            let addr = call_contract(client, chain, token_address, data).await?;

            Ok(U256::from((address!(&addr[26..]) == user_address) as u8))
        }
        None => {
            let data = format!("0x70a08231000000000000000000000000{addr}");
            let balance = call_contract(client, chain, token_address, data).await?;

            U256::from_str(&balance).map_err(|err| RpcError::Other(err.to_string()))
        }
    }
}

pub async fn get_erc1155_balance(
    client: &reqwest::Client,
    chain: EvmChain,
    token_address: Address,
    token_id: U256,
    user_address: Address,
) -> Result<U256, RpcError> {
    let addr = format!("{user_address:?}")[2..].to_string();
    let data = format!("0x00fdd58e000000000000000000000000{addr}{token_id:064x}");
    let balance = call_contract(client, chain, token_address, data).await?;

    U256::from_str(&balance).map_err(|err| RpcError::Other(err.to_string()))
}

#[cfg(all(test, feature = "nomock"))]
mod test {
    use crate::evm::{common::*, jsonrpc::get_erc20_decimals, EvmChain};
    use ethereum_types::U256;
    use rusty_gate_common::address;

    #[tokio::test]
    async fn rpc_get_erc20_decimals() {
        let client = reqwest::Client::new();
        let chain = EvmChain::Ethereum;
        let token_1 = ERC20_ADDR;
        let token_2 = "0x343e59d9d835e35b07fe80f5bb544f8ed1cd3b11";
        let token_3 = "0xaba8cac6866b83ae4eec97dd07ed254282f6ad8a";
        let token_4 = "0x0a9f693fce6f00a51a8e0db4351b5a8078b4242e";

        let decimals_1 = get_erc20_decimals(&client, chain, address!(token_1))
            .await
            .unwrap();
        let decimals_2 = get_erc20_decimals(&client, chain, address!(token_2))
            .await
            .unwrap();
        let decimals_3 = get_erc20_decimals(&client, chain, address!(token_3))
            .await
            .unwrap();
        let decimals_4 = get_erc20_decimals(&client, chain, address!(token_4))
            .await
            .unwrap();

        assert_eq!(decimals_1, U256::from(18));
        assert_eq!(decimals_2, U256::from(9));
        assert_eq!(decimals_3, U256::from(24));
        assert_eq!(decimals_4, U256::from(5));
    }
}
