//! A safe highly optimised alternative to libc `wmemchr`.
//!
//! This library provides optimised functions for the purpose of searching wide
//! character slices.
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! use wchar::wch;
//! use wmemchr::wmemchr;
//!
//! let needle = wch!('w');
//! let haystack = wch!("Hello world!");
//!
//! assert_eq!(wmemchr(needle, haystack), Some(6));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]
#![cfg_attr(feature = "unstable", feature(avx512_target_feature))]
#![deny(missing_docs)]

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

mod char;

pub mod fallback;
pub mod naive;

#[cfg(all(not(miri), target_arch = "x86_64"))]
pub mod x86_64;

pub use crate::char::Wide;

/// Returns the index of the first occurrence of a wide character in a slice,
/// or [`None`] if the character is not found.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use wchar::wch;
/// use wmemchr::wmemchr;
///
/// let haystack = wch!(u16, "foo bar");
///
/// assert_eq!(wmemchr(wch!(u16, 'o'), haystack), Some(1));
/// assert_eq!(wmemchr(wch!(u16, 'z'), haystack), None);
/// ```
#[inline]
pub fn wmemchr<T: Wide>(needle: T, haystack: &[T]) -> Option<usize> {
    cfg_if::cfg_if! {
        if #[cfg(miri)] {
            fallback::wmemchr(needle, haystack)
        } else if #[cfg(target_arch = "x86_64")] {
            x86_64::wmemchr(needle, haystack)
        } else {
            fallback::wmemchr(needle, haystack)
        }
    }
}
