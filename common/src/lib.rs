#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

pub use requirement::*;
use std::fmt;
pub use user::*;

mod requirement;
mod user;

#[derive(Clone, Copy)]
pub enum Chain {
    Ethereum,
    Polygon,
    Gnosis,
    Bsc,
    Goerli,
    Arbitrum,
    SolanaMain,
    SolanaTest,
    SolanaDev,
}

impl fmt::Debug for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match self {
            Self::Ethereum => "ethereum",
            Self::Polygon => "polygon",
            Self::Gnosis => "gnosis",
            Self::Bsc => "bsc",
            Self::Goerli => "goerli",
            Self::Arbitrum => "arbitrum",
            Self::SolanaMain => "solana_main",
            Self::SolanaTest => "solana_test",
            Self::SolanaDev => "solana_dev",
        };

        write!(f, "{res}")
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}
