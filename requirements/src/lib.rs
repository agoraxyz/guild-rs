#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

#[cfg(feature = "check")]
pub mod check;
pub mod relation;
pub mod token;

use guild_common::Scalar;
#[cfg(not(feature = "std"))]
use guild_common::{String, Vec};
use serde::{Deserialize, Serialize};
pub use serde_cbor::{from_slice as cbor_deserialize, to_vec as cbor_serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Requirement {
    pub prefix: u64,
    pub metadata: Vec<u8>,
    pub relation: relation::Relation<Scalar>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequirementsWithLogic {
    pub requirements: Vec<Requirement>,
    pub logic: String,
}

#[derive(Debug, Clone)]
pub struct SerializedRequirementsWithLogic {
    pub requirements: Vec<Vec<u8>>,
    pub logic: Vec<u8>,
}

impl TryFrom<RequirementsWithLogic> for SerializedRequirementsWithLogic {
    type Error = serde_cbor::Error;
    fn try_from(value: RequirementsWithLogic) -> Result<Self, Self::Error> {
        let requirements = value
            .requirements
            .into_iter()
            .map(|x| cbor_serialize(&x))
            .collect::<Result<Vec<_>, _>>()?;
        let logic = cbor_serialize(&value.logic)?;
        Ok(Self {
            requirements,
            logic,
        })
    }
}

impl TryFrom<SerializedRequirementsWithLogic> for RequirementsWithLogic {
    type Error = serde_cbor::Error;
    fn try_from(value: SerializedRequirementsWithLogic) -> Result<Self, Self::Error> {
        let requirements = value
            .requirements
            .into_iter()
            .map(|x| cbor_deserialize(&x))
            .collect::<Result<Vec<_>, _>>()?;
        let logic = cbor_deserialize(&value.logic)?;
        Ok(Self {
            requirements,
            logic,
        })
    }
}

#[cfg(test)]
mod test {
    use redis_test as _;
}
