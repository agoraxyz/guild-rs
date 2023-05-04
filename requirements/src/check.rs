use super::Requirement;
use guild_plugin_manager::redis::ConnectionLike;
use guild_plugin_manager::{CallOneInput, Client, PluginManager};

impl Requirement {
    pub fn check<C: ConnectionLike>(
        &self,
        mut pm: PluginManager<C>,
        client: Client,
        user: &[String],
    ) -> Result<bool, anyhow::Error> {
        let call_one_input = CallOneInput {
            client,
            user,
            serialized_secret: &pm.serialized_secret(self.prefix)?,
            serialized_metadata: &self.metadata,
        };

        let balances = pm.call_one(self.prefix, call_one_input)?;
        let balances_sum = balances.into_iter().sum();
        Ok(self.relation.assert(&balances_sum))
    }
}

#[cfg(test)]
mod test {
    use shiba as _;

    const ADDRESS_0: &str = "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE";
    const ADDRESS_1: &str = "0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3";
    const ADDRESS_2: &str = "0x283d678711daa088640c86a1ad3f12c00ec1252e";

    // TODO
    //"sol_pubkey": ["5MLhcU2vPXHwxUFXQJXYGQcFfetTthDajWf4CgSYtMK9"]
    //"sol_pubkey": ["4fYNw3dojWmQ4dXtSGE9epjRGy9pFSx62YypT7avPYvA"]
    //"sol_pubkey": ["vines1vzrYbzLMRdu58ou5XTby4qAqVRLmqo36NKPTg"]
    //let relation_2 = Relation::GreaterThan(420.0);
    //let sol_balance = Requirement {
    //    prefix: 1,
    //    metadata: vec![],
    //    relation: relation_2,
    //};
    // assert_eq!(
    //     sol_balance
    //         .check(&mut redis_cache, &client, &users)
    //         .unwrap(),
    //     vec![true, true, false]
    // );
    #[test]
    fn requirement_check() {
        let client = reqwest::Client::new();
        let token_type = TokenType::Fungible {
            address: "0x458691c1692cd82facfb2c5127e36d63213448a8".to_string(),
        };

        let relation = Relation::GreaterThan(0.0);

        let evm_balance = Requirement {
            prefix: 0,
            metadata: cbor_serialize(&token_type).unwrap(),
            relation,
        };

        let user = &[
            ADDRESS_0.to_string(),
            ADDRESS_1.to_string(),
            ADDRESS_2.to_string(),
        ];

        assert!(evm_balance.check(pm, client, user).unwrap());
    }
}
