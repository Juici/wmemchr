use crate::char::{KernelFn, Wide};

// TODO: Documentation.
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
