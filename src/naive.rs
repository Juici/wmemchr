//! A naive implementation.

use crate::char::{KernelFn, Wide};

/// Returns the index of the first occurrence of a wide character in a slice,
/// or [`None`] if the character is not found.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use wchar::wch;
/// use wmemchr::naive::wmemchr;
///
/// let haystack = wch!(u16, "foo bar");
///
/// assert_eq!(wmemchr(wch!(u16, 'o'), haystack), Some(1));
/// assert_eq!(wmemchr(wch!(u16, 'z'), haystack), None);
/// ```
#[inline(always)]
pub fn wmemchr<T: Wide>(needle: T, haystack: &[T]) -> Option<usize> {
    T::wmemchr_naive(needle, haystack)
}

pub(crate) struct Kernel;

impl<T: Copy + Eq> KernelFn<T> for Kernel {
    fn kernel(needle: T, haystack: &[T]) -> Option<usize> {
        haystack.iter().position(|&c| c == needle)
    }
}
