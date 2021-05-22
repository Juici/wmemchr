#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]
#![cfg_attr(feature = "unstable", feature(avx512_target_feature))]

#[cfg(feature = "unstable")]
extern crate std_detect;

#[macro_use]
mod macros;

mod char;

pub mod fallback;
pub mod naive;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub use crate::char::Wide;

// TODO: Documentation.
#[inline]
pub fn wmemchr<T: Wide>(needle: T, haystack: &[T]) -> Option<usize> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "x86_64")] {
            x86_64::wmemchr(needle, haystack)
        } else {
            fallback::wmemchr(needle, haystack)
        }
    }
}
