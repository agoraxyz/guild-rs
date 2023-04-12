use crate::{
    balance::contract::{Call, RpcError},
    rpc_error,
};
use guild_common::Scalar;
use primitive_types::U256;
use std::str::FromStr;

const FUNC_SIG: &str = "252dba42";
const PARAM_COUNT_LEN: usize = 32;
const DATA_PART_LEN: usize = 64;

pub fn aggregate(calls: &[Call]) -> String {
    let param_count_len = format!("{PARAM_COUNT_LEN:064x}");
    let param_count = format!("{:064x}", calls.len());

    let aggregated = calls
        .iter()
        .map(|call| {
            let data_len = call.call_data.len() / 2;
            let padding = vec!["0"; (DATA_PART_LEN - data_len) * 2].join("");

            format!(
                "{:0>64}{DATA_PART_LEN:064x}{data_len:064x}{}{padding}",
                call.target.trim_start_matches("0x"),
                call.call_data.trim_start_matches("0x")
            )
        })
        .collect::<String>();

    let offset = (0..(calls.len() * 5))
        .step_by(5)
        .map(|idx| format!("{:064x}", (idx + calls.len()) * 32))
        .collect::<String>();

    format!("{FUNC_SIG}{param_count_len}{param_count}{offset}{aggregated}")
}

pub fn parse_multicall_result(multicall_result: &str) -> Result<Vec<Scalar>, RpcError> {
    let lines = multicall_result
        .trim_start_matches("0x")
        .chars()
        .collect::<Vec<char>>()
        .chunks(64)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<String>>();

    let count = rpc_error!(U256::from_str(&lines[2]))?.as_usize();

    let balances = lines
        .into_iter()
        .skip(count + 4)
        .step_by(2)
        .map(|balance| rpc_error!(U256::from_str(&balance).map(|value| value.as_u128() as Scalar)))
        .collect::<Vec<Result<Scalar, RpcError>>>();

    balances.into_iter().collect()
}

#[cfg(test)]
mod test {
    use crate::balance::contract::{erc20_call, multicall::aggregate};

    #[test]
    fn aggregate_test() {
        let data = vec![
            // first 4 bytes of the keccak256 hash of the function signature
            // (balanceOf)
            "252dba42",
            // parameters count length (32 bytes)
            "0000000000000000000000000000000000000000000000000000000000000020",
            // parameters count (array length = 2)
            "0000000000000000000000000000000000000000000000000000000000000002",
            // offset of first parameter (64 bytes)
            "0000000000000000000000000000000000000000000000000000000000000040",
            // offset of second parameter (224 bytes)
            "00000000000000000000000000000000000000000000000000000000000000e0",
            // first element of the array
            // target contract address
            "000000000000000000000000458691c1692cd82facfb2c5127e36d63213448a8",
            // data part length (64 bytes)
            "0000000000000000000000000000000000000000000000000000000000000040",
            // data actual length (36 bytes)
            "0000000000000000000000000000000000000000000000000000000000000024",
            // data (erc20 balanceOf function signature, user address)
            "70a08231000000000000000000000000e43878ce78934fe8007748ff481f03b8",
            "ee3b97de00000000000000000000000000000000000000000000000000000000",
            // second element of the array
            // target contract address
            "000000000000000000000000458691c1692cd82facfb2c5127e36d63213448a8",
            // data part length (64 bytes)
            "0000000000000000000000000000000000000000000000000000000000000040",
            // data actual length (36 bytes)
            "0000000000000000000000000000000000000000000000000000000000000024",
            // data (erc20 balanceOf function signature, user address)
            "70a0823100000000000000000000000014ddfe8ea7ffc338015627d160ccaf99",
            "e8f16dd300000000000000000000000000000000000000000000000000000000",
        ]
        .join("");

        let erc20_addr = "0x458691c1692cd82facfb2c5127e36d63213448a8";
        let user1_addr = "0xe43878ce78934fe8007748ff481f03b8ee3b97de";
        let user2_addr = "0x14ddfe8ea7ffc338015627d160ccaf99e8f16dd3";

        let call_1 = erc20_call(erc20_addr, user1_addr);
        let call_2 = erc20_call(erc20_addr, user2_addr);

        assert_eq!(aggregate(&[call_1, call_2]), data);
    }
}
