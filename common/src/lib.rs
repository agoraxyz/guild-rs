#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

pub use requirement::*;
use std::fmt::Display;
pub use user::*;

mod requirement;
mod user;

#[derive(Debug)]
pub enum Chain {
    Ethereum,
    Polygon,
    Gnosis,
    Bsc,
    Goerli,
    Arbitrum,
}

impl Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lower = format!("{self:?}").to_lowercase();
        write!(f, "{lower}")
    }
}
