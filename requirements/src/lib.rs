#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

use guild_common::{Requirement, RequirementError};
pub use variants::*;

mod variants;
