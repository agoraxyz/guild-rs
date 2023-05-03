const FUNC_BALANCE_OF: &str = "70a08231";
const FUNC_DECIMALS: &str = "313ce567";
const FUNC_ERC1155_BATCH: &str = "4e1273f4";
const FUNC_ETH_BALANCE: &str = "4d2301cc";
const FUNC_OWNER_OF: &str = "6352211e";

#[derive(Clone)]
pub struct CallData(String);

impl CallData {
    pub fn new(raw: String) -> Self {
        Self(raw)
    }

    pub fn eth_balance(user_address: &str) -> Self {
        Self(format!(
            "{FUNC_ETH_BALANCE}{:0>64}",
            user_address.trim_start_matches("0x")
        ))
    }

    fn token_balance(user_address: &str) -> Self {
        Self(format!(
            "{FUNC_BALANCE_OF}{:0>64}",
            user_address.trim_start_matches("0x")
        ))
    }

    pub fn erc20_balance(user_address: &str) -> Self {
        Self::token_balance(user_address)
    }

    pub fn erc721_balance(user_address: &str) -> Self {
        Self::token_balance(user_address)
    }

    pub fn erc721_owner(id: &str) -> Self {
        Self(format!("{FUNC_OWNER_OF}{id:0>64}"))
    }

    pub fn erc1155_balance_batch(user_addresses: &[String], token_id: &str) -> Self {
        let padded_addresses = user_addresses
            .iter()
            .map(|address| format!("{:0>64}", address.trim_start_matches("0x")))
            .collect::<String>();

        let len = 64;
        let n_addresses = padded_addresses.len();
        let offset = (n_addresses + 3) * 32;
        let token_ids = vec![format!("{token_id:0>64}"); n_addresses].join("");

        Self(format!(
            "{FUNC_ERC1155_BATCH}{len:064x}{offset:064x}{n_addresses:064x}{padded_addresses}{n_addresses:064x}{token_ids}",
        ))
    }

    pub fn erc20_decimals() -> Self {
        Self(FUNC_DECIMALS.to_string())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn raw(&self) -> &str {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::CallData;

    const TEST_ADDRESS: &str = "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE";

    #[test]
    fn eth_balance() {
        let call_data = CallData::eth_balance(TEST_ADDRESS);
        assert_eq!(
            call_data.raw(),
            "4d2301cc000000000000000000000000E43878Ce78934fe8007748FF481f03B8Ee3b97DE"
        );
    }
}
