use guild_common::Scalar;
use primitive_types::U256;

use std::str::FromStr;

pub struct Balances(Vec<Scalar>);

impl Balances {
    pub fn new(inner: Vec<Scalar>) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> Vec<Scalar> {
        self.0
    }

    pub fn normalize(&mut self, decimals: u32) {
        self.0
            .iter_mut()
            .for_each(|balance| *balance /= 10u128.pow(decimals) as Scalar)
    }

    pub fn from_response(response: &str) -> Result<Self, anyhow::Error> {
        let lines = response
            .trim_start_matches("0x")
            .as_bytes()
            .chunks(64)
            .map(|chunk| {
                std::str::from_utf8(chunk)
                    .expect("original string already contains valid utf8")
                    .to_string()
            })
            .collect::<Vec<String>>();

        let count =
            U256::from_str(&lines.get(2).ok_or(anyhow::anyhow!("invalid input"))?)?.as_usize();

        let balances = lines
            .into_iter()
            .skip(count + 4)
            .step_by(2)
            .map(|line| {
                U256::from_str(&line)
                    .map(|value| value.as_u128() as Scalar)
                    .map_err(|e| anyhow::anyhow!(e))
            })
            .collect::<Result<Vec<Scalar>, anyhow::Error>>()?;

        Ok(Self(balances))
    }

    pub fn from_special_response(response: &str) -> Result<Self, anyhow::Error> {
        let balances = response
            .trim_start_matches("0x")
            .as_bytes()
            .chunks(64)
            .skip(2)
            .map(|chunk| {
                let line = std::str::from_utf8(chunk)
                    .expect("original string already contains valid utf8");
                U256::from_str(&line)
                    .map(|value| value.as_u128() as Scalar)
                    .map_err(|e| anyhow::anyhow!(e))
            })
            .collect::<Result<Vec<Scalar>, anyhow::Error>>()?;
        Ok(Self(balances))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn normalize() {
        let mut balances = Balances::new(vec![0.0, 1.0, 2.0, 100.0, 1000.0]);
        balances.normalize(2);
        assert_eq!(balances.into_inner(), vec![0.0, 0.01, 0.02, 1.0, 10.0]);
    }

    #[test]
    fn parse() {
        let input = vec![
            "0x",
            // 1st line (line 0)
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000001",
            // 3rd line with count (10)
            "000000000000000000000000000000000000000000000000000000000000000a",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
            "0000000000000000000000000000000000000000000000000000000000000005",
            "0000000000000000000000000000000000000000000000000000000000000006",
            "0000000000000000000000000000000000000000000000000000000000000007",
            "0000000000000000000000000000000000000000000000000000000000000008",
            "0000000000000000000000000000000000000000000000000000000000000009",
            "0000000000000000000000000000000000000000000000000000000000000010",
            "0000000000000000000000000000000000000000000000000000000000000011",
            "0000000000000000000000000000000000000000000000000000000000000012",
            "0000000000000000000000000000000000000000000000000000000000000013",
            // skip 14 lines (and step by 2)
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000015",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000017",
            "0000000000000000000000000000000000000000000000000000000000000010",
            "0000000000000000000000000000000000000000000000000000000000000019",
            "0000000000000000000000000000000000000000000000000000000000000100",
            "0000000000000000000000000000000000000000000000000000000000000021",
            "0000000000000000000000000000000000000000000000000000000000001000",
            "0000000000000000000000000000000000000000000000000000000000000023",
            "0000000000000000000000000000000000000000000000000000000000010000",
            "0000000000000000000000000000000000000000000000000000000000000025",
            "0000000000000000000000000000000000000000000000000000000000001000",
            "0000000000000000000000000000000000000000000000000000000000000027",
            "0000000000000000000000000000000000000000000000000000000000000100",
            "0000000000000000000000000000000000000000000000000000000000000029",
            "0000000000000000000000000000000000000000000000000000000000000010",
            "0000000000000000000000000000000000000000000000000000000000000031",
            "0000000000000000000000000000000000000000000000000000000000000001",
        ]
        .join("");
        let balances = Balances::from_response(&input).unwrap();
        assert_eq!(
            balances.into_inner(),
            &[0.0, 1.0, 16.0, 256.0, 4096.0, 65536.0, 4096.0, 256.0, 16.0, 1.0]
        );

        assert!(Balances::from_response("").is_err());
    }
}
