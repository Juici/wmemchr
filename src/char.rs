use crate::fallback;
use crate::naive;

#[cfg(all(not(miri), target_arch = "x86_64"))]
use crate::x86_64;

mod private {
    pub trait Sealed {}
}

/// A `wmemchr` kernel.
pub(crate) trait KernelFn<T> {
    /// The kernel function.
    fn kernel(needle: T, haystack: &[T]) -> Option<usize>;
}

/// A trait for wide character types.
pub trait Wide: private::Sealed + Copy + Eq + 'static {
    #[doc(hidden)]
    fn wmemchr_naive(needle: Self, haystack: &[Self]) -> Option<usize>;
    #[doc(hidden)]
    fn wmemchr_fallback(needle: Self, haystack: &[Self]) -> Option<usize>;
    #[doc(hidden)]
    #[cfg(all(not(miri), target_arch = "x86_64"))]
    fn wmemchr_x86_64(needle: Self, haystack: &[Self]) -> Option<usize>;
}

macro_rules! impl_wide {
    ($($ty:ty),*) => {
        $(
            impl private::Sealed for $ty {}

            impl Wide for $ty {
                #[inline(always)]
                fn wmemchr_naive(needle: $ty, haystack: &[$ty]) -> Option<usize> {
                    naive::Kernel::kernel(needle, haystack)
                }
                #[inline(always)]
                fn wmemchr_fallback(needle: $ty, haystack: &[$ty]) -> Option<usize> {
                    fallback::Kernel::kernel(needle, haystack)
                }
                #[inline(always)]
                #[cfg(all(not(miri), target_arch = "x86_64"))]
                fn wmemchr_x86_64(needle: $ty, haystack: &[$ty]) -> Option<usize> {
                    x86_64::Kernel::kernel(needle, haystack)
                }
            }
        )*
    };
}
impl_wide!(u16, u32, i16, i32);
