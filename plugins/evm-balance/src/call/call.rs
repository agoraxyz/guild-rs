use super::CallData;

use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct Response {
    pub result: String,
}

pub struct Call {
    target: String,
    call_data: CallData,
}

impl Call {
    pub fn new(target: String, call_data: CallData) -> Self {
        Self { target, call_data }
    }

    pub fn dispatch(self, client: Client, rpc_url: &str) -> Result<String, anyhow::Error> {
        let params = json!([
            {
                "to"   : self.target,
                "data" : format!("0x{}", self.call_data.raw())
            },
            "latest"
        ]);

        let payload = create_payload("eth_call", params, 1);

        let response: Response = client.post(rpc_url).json(&payload).send()?.json()?;

        Ok(response.result)
    }

    #[cfg(test)]
    pub fn target(&self) -> &str {
        &self.target
    }

    #[cfg(test)]
    pub fn call_data(&self) -> &CallData {
        &self.call_data
    }
}

fn create_payload(method: &str, params: Value, id: u32) -> Value {
    json!({
        "method"  : method,
        "params"  : params,
        "id"      : id,
        "jsonrpc" : "2.0"
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use primitive_types::U256;

    use std::str::FromStr;

    const RPC_URL: &str = "https://eth.public-rpc.com";

    #[test]
    fn get_erc20_decimals() {
        // arrange
        let client = Client::new();
        let tokens = vec![
            "0x458691c1692CD82faCfb2C5127e36D63213448A8".to_string(),
            "0x343e59d9d835e35b07fe80f5bb544f8ed1cd3b11".to_string(),
            "0xaba8cac6866b83ae4eec97dd07ed254282f6ad8a".to_string(),
            "0x0a9f693fce6f00a51a8e0db4351b5a8078b4242e".to_string(),
        ];
        let call_data = CallData::erc20_decimals();
        let expected = vec![18u64, 9, 24, 5]
            .into_iter()
            .map(U256::from)
            .collect::<Vec<U256>>();

        // act
        let result_strings = tokens
            .into_iter()
            .map(|token| Call::new(token, call_data.clone()).dispatch(client.clone(), RPC_URL))
            .collect::<Result<Vec<String>, anyhow::Error>>()
            .unwrap();

        let decimals = result_strings
            .into_iter()
            .map(|result_string| U256::from_str(&result_string).unwrap())
            .collect::<Vec<U256>>();

        // assert
        assert_eq!(decimals, expected);
    }
}
