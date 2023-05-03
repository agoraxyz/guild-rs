use super::calldata::CallData;
use super::Call;

const DATA_PART_LEN: usize = 64;
const FUNC_SIG: &str = "252dba42";
const PARAM_COUNT_LEN: usize = 32;

pub struct Multicall(Vec<CallData>);

impl Multicall {
    pub fn eth_balances(user_addresses: &[String]) -> Self {
        Self(
            user_addresses
                .iter()
                .map(|address| CallData::eth_balance(address))
                .collect::<Vec<CallData>>(),
        )
    }

    pub fn erc20_balances(user_addresses: &[String]) -> Self {
        Self(
            user_addresses
                .iter()
                .map(|address| CallData::erc20_balance(address))
                .collect::<Vec<CallData>>(),
        )
    }

    pub fn erc721_balances(user_addresses: &[String]) -> Self {
        Self(
            user_addresses
                .iter()
                .map(|address| CallData::erc721_balance(address))
                .collect::<Vec<CallData>>(),
        )
    }

    pub fn aggregate(self, target: String, contract: String) -> Call {
        let n_calls = self.0.len();
        let param_count_len = format!("{PARAM_COUNT_LEN:064x}");
        let param_count = format!("{:064x}", n_calls);

        let aggregated = self
            .0
            .into_iter()
            .map(|call_data| {
                let data_len = call_data.len() / 2;
                let padding = vec!["0"; (DATA_PART_LEN - data_len) * 2].join("");

                format!(
                    "{:0>64}{DATA_PART_LEN:064x}{data_len:064x}{}{padding}",
                    contract.as_str().trim_start_matches("0x"),
                    call_data.raw().trim_start_matches("0x")
                )
            })
            .collect::<String>();

        let offset = (0..(n_calls * 5))
            .step_by(5)
            .map(|idx| format!("{:064x}", (idx + n_calls) * 32))
            .collect::<String>();

        Call::new(
            target,
            CallData::new(format!(
                "{FUNC_SIG}{param_count_len}{param_count}{offset}{aggregated}"
            )),
        )
    }
}

#[cfg(test)]
mod test {
    use super::Multicall;

    const TEST_ADDRESS: &str = "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE";

    #[test]
    fn eth_aggregation() {
        let expected = vec![
            "252dba42",
            "0000000000000000000000000000000000000000000000000000000000000020",
            "0000000000000000000000000000000000000000000000000000000000000005",
            "00000000000000000000000000000000000000000000000000000000000000a0",
            "0000000000000000000000000000000000000000000000000000000000000140",
            "00000000000000000000000000000000000000000000000000000000000001e0",
            "0000000000000000000000000000000000000000000000000000000000000280",
            "0000000000000000000000000000000000000000000000000000000000000320",
            "0000000000000000000000005BA1e12693Dc8F9c48aAD8770482f4739bEeD696",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000024",
            "4d2301cc000000000000000000000000E43878Ce78934fe8007748FF481f03B8",
            "Ee3b97DE00000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000005BA1e12693Dc8F9c48aAD8770482f4739bEeD696",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000024",
            "4d2301cc000000000000000000000000E43878Ce78934fe8007748FF481f03B8",
            "Ee3b97DE00000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000005BA1e12693Dc8F9c48aAD8770482f4739bEeD696",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000024",
            "4d2301cc000000000000000000000000E43878Ce78934fe8007748FF481f03B8",
            "Ee3b97DE00000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000005BA1e12693Dc8F9c48aAD8770482f4739bEeD696",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000024",
            "4d2301cc000000000000000000000000E43878Ce78934fe8007748FF481f03B8",
            "Ee3b97DE00000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000005BA1e12693Dc8F9c48aAD8770482f4739bEeD696",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000024",
            "4d2301cc000000000000000000000000E43878Ce78934fe8007748FF481f03B8",
            "Ee3b97DE00000000000000000000000000000000000000000000000000000000",
        ]
        .join("");

        let contract = "5BA1e12693Dc8F9c48aAD8770482f4739bEeD696";
        let target = "target";
        let addresses = vec![TEST_ADDRESS.to_string(); 5];
        let multicall = Multicall::eth_balances(&addresses);
        let call = multicall.aggregate(target.to_string(), contract.to_string());
        assert_eq!(call.target(), target);
        assert_eq!(call.call_data().raw(), expected);
    }

    #[test]
    fn erc20_aggregation() {
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

        let target = "target";
        let erc20_addr = "0x458691c1692cd82facfb2c5127e36d63213448a8".to_string();
        let user1_addr = "0xe43878ce78934fe8007748ff481f03b8ee3b97de".to_string();
        let user2_addr = "0x14ddfe8ea7ffc338015627d160ccaf99e8f16dd3".to_string();

        let multicall = Multicall::erc20_balances(&[user1_addr, user2_addr]);
        let call = multicall.aggregate(target.to_string(), erc20_addr);

        assert_eq!(call.target(), target);
        assert_eq!(call.call_data().raw(), data);
    }
}
