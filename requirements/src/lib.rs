#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]

#[cfg(test)]
use rusty_gate_common::evm_addr;
use rusty_gate_common::{Address, Identity, Requirement, RequirementError, User, U256};

mod variants;
