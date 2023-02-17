use super::{Call, ZEROES};
use ethereum_types::U256;
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
                "{ZEROES}{:x}{DATA_PART_LEN:064x}{data_len:064x}{}{padding}",
                call.target, call.call_data
            )
        })
        .collect::<String>();

    let offset = (0..(calls.len() * 5))
        .step_by(5)
        .map(|idx| format!("{:064x}", (idx + calls.len()) * 32))
        .collect::<String>();

    format!("{FUNC_SIG}{param_count_len}{param_count}{offset}{aggregated}")
}

pub fn parse_multicall_result(multicall_result: &str) -> Vec<U256> {
    let lines = multicall_result
        .chars()
        .skip(2)
        .collect::<Vec<char>>()
        .chunks(64)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<String>>();

    let count = U256::from_str(&lines[2]).unwrap_or_default().as_usize();

    lines
        .iter()
        .skip(count + 4)
        .step_by(2)
        .map(|balance| U256::from_str(balance).unwrap_or_default())
        .collect::<Vec<U256>>()
}

#[cfg(test)]
mod test {
    use crate::evm::jsonrpc::contract::{erc20_call, multicall::aggregate};
    use rusty_gate_common::address;

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

        let call_1 = erc20_call(
            address!(erc20_addr),
            address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"),
        );
        let call_2 = erc20_call(
            address!(erc20_addr),
            address!("0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3"),
        );

        assert_eq!(aggregate(&[call_1, call_2]), data);
    }
}
