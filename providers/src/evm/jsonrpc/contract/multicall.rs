use super::Call;

const FUNC_SIG: &str = "252dba42";
const PARAM_COUNT_LEN: usize = 32;
const ZEROES: &str = "000000000000000000000000";
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

#[cfg(test)]
mod test {
    use crate::evm::jsonrpc::contract::{erc20_call, multicall::aggregate};
    use rusty_gate_common::address;

    #[test]
    fn aggregate_test() {
        let data = vec![
            // function signature
            "252dba42",
            // parameters count length (32 bytes)
            "0000000000000000000000000000000000000000000000000000000000000020",
            // parameters count (1)
            "0000000000000000000000000000000000000000000000000000000000000001",
            // offset of first parameter ((address,bytes))
            "0000000000000000000000000000000000000000000000000000000000000020",
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

        let call = erc20_call(
            address!("0x458691c1692cd82facfb2c5127e36d63213448a8"),
            address!("0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3"),
        );

        let call_data = aggregate(&vec![call]);

        assert_eq!(call_data, data);
    }
}
