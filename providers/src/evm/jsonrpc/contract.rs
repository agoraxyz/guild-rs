use crate::{
    evm::jsonrpc::{create_payload, types::RpcResponse, JsonRpcMethods, RpcError, ETHEREUM},
    CLIENT,
};
use ethereum_types::{Address, U256};

pub async fn call_contract(
    contract_address: Address,
    user_address: Address,
) -> Result<U256, RpcError> {
    let addr = format!("{user_address:?}")[2..].to_string();
    let params = format!(
        "[
            {{
                \"to\": \"{contract_address:?}\",
                \"data\": \"0x70a08231000000000000000000000000{addr}\"
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

#[cfg(test)]
mod test {
    use super::call_contract;
    use ethereum_types::U256;
    use rusty_gate_common::address;

    #[tokio::test]
    async fn call_contract_test() {
        assert_eq!(
            call_contract(
                address!("0x458691c1692cd82facfb2c5127e36d63213448a8"),
                address!("0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3")
            )
            .await
            .unwrap(),
            U256::from(100000000000000000000_u128)
        );
    }
}
