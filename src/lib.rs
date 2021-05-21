#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]

#[cfg(feature = "unstable")]
extern crate std_detect;

#[macro_use]
mod macros;

mod char;
mod kernel;

pub mod fallback;
pub mod naive;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub use crate::char::Wide;
