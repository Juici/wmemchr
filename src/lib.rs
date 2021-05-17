#![cfg_attr(not(feature = "std"), no_std)]

mod char;

pub mod naive;
pub mod fallback;
pub mod simd;

pub use crate::char::Wide;
