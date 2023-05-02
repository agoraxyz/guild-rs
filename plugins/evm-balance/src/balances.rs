use guild_common::Scalar;
use primitive_types::U256;

use std::str::FromStr;

pub struct Balances(Vec<Scalar>);

impl Balances {
    pub fn into_inner(self) -> Vec<Scalar> {
        self.0
    }
}

impl FromStr for Balances {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s
            .trim_start_matches("0x")
            .chars()
            .collect::<Vec<char>>()
            .chunks(64)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<String>>();

        let count = U256::from_str(&lines[2])?.as_usize();

        let inner = lines
            .into_iter()
            .skip(count + 4)
            .step_by(2)
            .map(|balance| {
                U256::from_str(&balance)
                    .map(|value| value.as_u128() as Scalar)
                    .map_err(|e| anyhow::anyhow!(e))
            })
            .collect::<Result<Vec<Scalar>, anyhow::Error>>()?;

        Ok(Self(inner))
    }
}
