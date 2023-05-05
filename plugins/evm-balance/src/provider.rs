use crate::balances::Balances;
use crate::call::*;
use guild_requirements::token::TokenType;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Provider {
    pub rpc_url: String,
    pub multicall_contract: String,
}

impl Provider {
    pub fn balances(
        self,
        client: Client,
        token_type: TokenType,
        addresses: &[String],
    ) -> Result<Balances, anyhow::Error> {
        let Provider {
            mut rpc_url,
            multicall_contract,
        } = self;
        let balances = match token_type {
            TokenType::Native => eth_balances(client, addresses, multicall_contract, &rpc_url),
            TokenType::Fungible { address } => {
                erc20_balances(client, addresses, multicall_contract, address, &rpc_url)
            }
            TokenType::NonFungible {
                address,
                id: maybe_id,
            } => match maybe_id {
                Some(id) => erc721_ownership(client, addresses, address, id, &rpc_url),
                None => erc721_balances(client, addresses, multicall_contract, address, &rpc_url),
            },
            TokenType::Special {
                address,
                id: maybe_id,
            } => match maybe_id {
                Some(id) => erc1155_balances(client, addresses, address, id, &rpc_url),
                None => Ok(Balances::new(vec![0.0; addresses.len()])),
            },
        };
        rpc_url.zeroize();
        balances
    }
}

#[cfg(test)]
mod test {
    use super::{Provider, TokenType, Client};

    const USER_1_ADDR: &str = "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE";
    const USER_2_ADDR: &str = "0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3";
    const USER_3_ADDR: &str = "0x283D678711dAa088640C86a1ad3f12C00EC1252E";
    const ERC20_ADDR: &str = "0x458691c1692CD82faCfb2C5127e36D63213448A8";
    const ERC721_ADDR: &str = "0x57f1887a8BF19b14fC0dF6Fd9B2acc9Af147eA85";
    const ERC721_ID: &str =
        "61313325075603536901663283754390960556726744542208800735045237225934362163454";
    const ERC1155_ADDR: &str = "0x76BE3b62873462d2142405439777e971754E8E77";
    const ERC1155_ID: usize = 10868;
    const MULTICALL_CONTRACT: &str = "0x5BA1e12693Dc8F9c48aAD8770482f4739bEeD696";
    const RPC_URL: &str = "https://eth.public-rpc.com";

    fn dummy() -> (Client, Provider) {
        (
            Client::new(),
            Provider {
                rpc_url: RPC_URL.to_string(),
                multicall_contract: MULTICALL_CONTRACT.to_string(),
            },
        )
    }

    #[test]
    fn eth_balances() {
        let (client, provider) = dummy();
        assert_eq!(
            provider
                .balances(
                    client,
                    TokenType::Native,
                    &[USER_1_ADDR.to_string(), USER_2_ADDR.to_string()]
                )
                .unwrap()
                .into_inner(),
            vec![0.000464468855704627, 0.3919455024496939]
        );
    }

    #[test]
    fn erc20_balances() {
        let (client, provider) = dummy();
        let token_type = TokenType::Fungible {
            address: ERC20_ADDR.to_string(),
        };

        assert_eq!(
            provider
                .balances(
                    client,
                    token_type,
                    &[USER_1_ADDR.to_string(), USER_2_ADDR.to_string()]
                )
                .unwrap()
                .into_inner(),
            vec![0.0, 100.0]
        );
    }

    #[test]
    fn erc721_without_id() {
        let (client, provider) = dummy();
        let token_type_without_id = TokenType::NonFungible {
            address: ERC721_ADDR.to_string(),
            id: None,
        };
        assert_eq!(
            provider
                .balances(
                    client.clone(),
                    token_type_without_id,
                    &[USER_1_ADDR.to_string(), USER_2_ADDR.to_string()]
                )
                .unwrap()
                .into_inner(),
            vec![1.0, 1.0]
        );
    }

    #[test]
    fn erc721_with_id() {
        let (client, provider) = dummy();
        let token_type_with_id = TokenType::NonFungible {
            address: ERC721_ADDR.to_string(),
            id: Some(ERC721_ID.to_string()),
        };
        assert_eq!(
            provider
                .balances(
                    client,
                    token_type_with_id,
                    &[USER_1_ADDR.to_string(), USER_2_ADDR.to_string()]
                )
                .unwrap()
                .into_inner(),
            vec![1.0, 0.0]
        );
    }

    #[test]
    fn erc1155_without_id() {
        let (client, provider) = dummy();
        let token_type_with_id = TokenType::Special {
            address: ERC1155_ADDR.to_string(),
            id: None,
        };

        assert_eq!(
            provider
                .balances(
                    client,
                    token_type_with_id,
                    &[USER_1_ADDR.to_string(), USER_3_ADDR.to_string()]
                )
                .unwrap()
                .into_inner(),
            vec![0.0, 0.0]
        );
    }

    #[test]
    fn erc1155_with_id() {
        let (client, provider) = dummy();
        let token_type_with_id = TokenType::Special {
            address: ERC1155_ADDR.to_string(),
            id: Some(ERC1155_ID.to_string()),
        };

        assert_eq!(
            provider
                .balances(
                    client,
                    token_type_with_id,
                    &[USER_1_ADDR.to_string(), USER_3_ADDR.to_string()]
                )
                .unwrap()
                .into_inner(),
            vec![0.0, 15.0]
        );
    }
}
