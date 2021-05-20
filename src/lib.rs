#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
mod macros;

mod char;
mod kernel;

pub mod fallback;
pub mod naive;
mod x86;

pub use crate::char::Wide;
