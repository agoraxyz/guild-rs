use crate::evm::{
    jsonrpc::{contract::multicall::aggregate, create_payload, GetProvider, RpcError, RpcResponse},
    EvmChain,
};
use ethereum_types::{Address, U256};
use rusty_gate_common::address;
use std::str::FromStr;

mod multicall;

const ZEROES: &str = "000000000000000000000000";

#[derive(Clone, Debug)]
pub struct Call {
    pub target: Address,
    pub call_data: String,
}

async fn call_contract(
    client: &reqwest::Client,
    chain: EvmChain,
    call: Call,
) -> Result<String, RpcError> {
    let provider = chain
        .provider()
        .map_err(|err| RpcError::Other(err.to_string()))?;

    let params = format!(
        "[
            {{
                \"to\"   : \"{:?}\",
                \"data\" : \"0x{}\"
            }},
            \"latest\"
        ]",
        call.target, call.call_data
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

pub async fn get_eth_balance_batch(
    client: &reqwest::Client,
    chain: EvmChain,
    user_addresses: &[Address],
) -> Result<Vec<U256>, RpcError> {
    let target = chain.provider()?.contract;

    let calls = user_addresses
        .iter()
        .map(|addr| Call {
            target,
            call_data: format!("4d2301cc{ZEROES}{addr:x}"),
        })
        .collect::<Vec<Call>>();

    let call = Call {
        target,
        call_data: aggregate(&calls),
    };

    let _balance = dbg!(call_contract(client, chain, call).await?);

    Ok(vec![])
}

pub async fn get_erc20_decimals(
    client: &reqwest::Client,
    chain: EvmChain,
    token_address: Address,
) -> Result<U256, RpcError> {
    let call = Call {
        target: token_address,
        call_data: "313ce567".to_string(),
    };
    let decimals = call_contract(client, chain, call).await?;

    U256::from_str(&decimals).map_err(|err| RpcError::Other(err.to_string()))
}

fn erc20_call(token_address: Address, user_address: Address) -> Call {
    Call {
        target: token_address,
        call_data: format!("70a08231{ZEROES}{user_address:x}"),
    }
}

pub async fn get_erc20_balance(
    client: &reqwest::Client,
    chain: EvmChain,
    token_address: Address,
    user_address: Address,
) -> Result<U256, RpcError> {
    let call = erc20_call(token_address, user_address);
    let balance = call_contract(client, chain, call).await?;

    U256::from_str(&balance).map_err(|err| RpcError::Other(err.to_string()))
}

pub async fn get_erc20_balance_batch(
    client: &reqwest::Client,
    chain: EvmChain,
    token_address: Address,
    user_addresses: &[Address],
) -> Result<Vec<U256>, RpcError> {
    let calls = user_addresses
        .iter()
        .map(|user_address| erc20_call(token_address, *user_address))
        .collect::<Vec<Call>>();

    let call = Call {
        target: chain.provider()?.contract,
        call_data: aggregate(&calls),
    };

    let _balance = dbg!(call_contract(client, chain, call).await?);

    Ok(vec![])
}

pub fn erc721_call(token_address: Address, user_address: Address) -> Call {
    Call {
        target: token_address,
        call_data: format!("70a08231{ZEROES}{user_address:x}"),
    }
}

fn erc721_id_call(token_address: Address, id: U256) -> Call {
    let call_data = format!("6352211e{id:064x}");

    Call {
        target: token_address,
        call_data,
    }
}

pub async fn get_erc721_balance(
    client: &reqwest::Client,
    chain: EvmChain,
    token_address: Address,
    token_id: Option<U256>,
    user_address: Address,
) -> Result<U256, RpcError> {
    match token_id {
        Some(id) => {
            let call = erc721_id_call(token_address, id);
            let addr = call_contract(client, chain, call).await?;

            Ok(U256::from((address!(&addr[26..]) == user_address) as u8))
        }
        None => {
            let call = erc721_call(token_address, user_address);
            let balance = call_contract(client, chain, call).await?;

            U256::from_str(&balance).map_err(|err| RpcError::Other(err.to_string()))
        }
    }
}

pub async fn get_erc721_balance_batch(
    client: &reqwest::Client,
    chain: EvmChain,
    token_address: Address,
    user_addresses: &[Address],
) -> Result<Vec<U256>, RpcError> {
    let calls = user_addresses
        .iter()
        .map(|user_address| erc721_call(token_address, *user_address))
        .collect::<Vec<Call>>();

    let call = Call {
        target: chain.provider()?.contract,
        call_data: aggregate(&calls),
    };

    let _balance = dbg!(call_contract(client, chain, call).await?);

    Ok(vec![])
}

fn erc1155_call(token_address: Address, id: U256, user_address: Address) -> Call {
    Call {
        target: token_address,
        call_data: format!("00fdd58e{ZEROES}{user_address:x}{id:064x}"),
    }
}

pub async fn get_erc1155_balance(
    client: &reqwest::Client,
    chain: EvmChain,
    token_address: Address,
    token_id: U256,
    user_address: Address,
) -> Result<U256, RpcError> {
    let call = erc1155_call(token_address, token_id, user_address);
    let balance = call_contract(client, chain, call).await?;

    U256::from_str(&balance).map_err(|err| RpcError::Other(err.to_string()))
}

pub async fn get_erc1155_balance_batch(
    client: &reqwest::Client,
    chain: EvmChain,
    token_address: Address,
    token_id: U256,
    user_addresses: &[Address],
) -> Result<Vec<U256>, RpcError> {
    let addresses = user_addresses
        .iter()
        .map(|user_address| format!("{ZEROES}{user_address:x}"))
        .collect::<String>();

    let len = 64;
    let count = user_addresses.len();
    let ids = vec![format!("{token_id:064x}"); count].join("");
    let offset = (count + 3) * 32;
    let call_data =
        format!("4e1273f4{len:064x}{offset:064x}{count:064x}{addresses}{count:064x}{ids}",);

    let call = Call {
        target: token_address,
        call_data,
    };

    let _balance = dbg!(call_contract(client, chain, call).await?);

    Ok(vec![])
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
