#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use guild_common::{Requirement, RequirementResult, User};
use libloading::{Library, Symbol};
use reqwest::Client;

const LIB_PATH: &str = "../requirements/evm_balance/target/release/libevm_balance.dylib";

pub trait Checkable {
    fn check(&self, client: &Client, users: &[User]) -> RequirementResult;
}

impl Checkable for Requirement {
    fn check(&self, client: &Client, users: &[User]) -> RequirementResult {
        let lib = unsafe { Library::new(LIB_PATH) }.unwrap();

        let check_req: Symbol<extern "C" fn(&Client, &[User], &str, &str) -> RequirementResult> =
            unsafe { lib.get(b"check") }.unwrap();

        let secrets = r#"{
          "rpc_url": "https://eth.public-rpc.com",
          "contract": "0x5ba1e12693dc8f9c48aad8770482f4739beed696",
          "balancy_id": 1
        }"#;

        check_req(client, &users, &self.metadata, secrets)
    }
}

#[cfg(test)]
mod test {
    use super::{Checkable, Requirement, RequirementResult, User};
    use guild_common::{Relation, RequirementType, TokenType};
    use reqwest::Client;
    use tokio::runtime;

    const USERS: &str = r#"[
    {
        "id": 0,
        "identities": {
            "evm_address": ["0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"]
        }
    },
    {
        "id": 1,
        "identities": {
            "evm_address": ["0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3"]
        }
    },
    {
        "id": 2,
        "identities": {
            "evm_address": ["0x283d678711daa088640c86a1ad3f12c00ec1252e"]
        }
    }
    ]"#;

    #[test]
    fn requirement_check() {
        let token_type = TokenType::Special {
            address: "0x76be3b62873462d2142405439777e971754e8e77".to_string(),
            id: None,
        };
        let relation = Relation::GreaterThan(0.0);

        let req = Requirement {
            id: "69".to_string(),
            typ: RequirementType::EvmBalance,
            metadata: serde_json::to_string(&(token_type, relation)).unwrap(),
        };

        let client = Client::new();
        let users: Vec<User> = serde_json::from_str(USERS).unwrap();

        let rt = runtime::Runtime::new().unwrap();

        rt.block_on(async {
            assert_eq!(
                req.check(&client, &users),
                RequirementResult::Ok(vec![false, false, true])
            );
        });
    }
}
