#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

pub type Scalar = f64;

#[cfg(feature = "std")]
use parity_scale_codec as _;

#[cfg(not(feature = "std"))]
pub use parity_scale_codec::alloc::string::String;
#[cfg(not(feature = "std"))]
pub use parity_scale_codec::alloc::vec::Vec;
