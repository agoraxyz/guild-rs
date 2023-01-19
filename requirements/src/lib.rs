#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]

use ethereum_types::{Address, U256};
#[cfg(test)]
use rusty_gate_common::evm_addr;
use rusty_gate_common::{Identity, Requirement, RequirementError, User};

mod variants;
