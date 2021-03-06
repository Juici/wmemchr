//! Optimised implementation for x86_64 platforms.

use crate::char::{KernelFn, Wide};

mod avx2;
#[cfg(feature = "unstable")]
mod evex;
mod sse2;

/// Returns the index of the first occurrence of a wide character in a slice,
/// or [`None`] if the character is not found.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # fn main() {
/// # #[cfg(not(miri))]
/// # {
/// use wchar::wch;
/// use wmemchr::x86_64::wmemchr;
///
/// let haystack = wch!(u16, "foo bar");
///
/// assert_eq!(wmemchr(wch!(u16, 'o'), haystack), Some(1));
/// assert_eq!(wmemchr(wch!(u16, 'z'), haystack), None);
/// # }
/// # }
/// ```
#[inline(always)]
pub fn wmemchr<T: Wide>(needle: T, haystack: &[T]) -> Option<usize> {
    T::wmemchr_x86_64(needle, haystack)
}

macro_rules! unsafe_ifuncs {
    ($(
        fn $ty:ident::$name:ident($($arg:ident: $arg_ty:ty),* $(,)?) $(-> $ret_ty:ty)?;
    )*) => {
        $(
            unsafe_ifuncs! {
                @__item
                [$ty]
                [$name]
                [$($arg: $arg_ty),*]
                [$($ret_ty)?]
                [fn($($arg_ty),*) $(-> $ret_ty)?]
            }
        )*
    };
    (@__item [$ty:ident] [$name:ident] [$($arg:ident: $arg_ty:ty),*] [$($ret_ty:ty)?] [$fn_ty:ty]) => {
        mod $ty {
            use core::mem;
            use core::sync::atomic::{AtomicPtr, Ordering};

            type FnRaw = *mut ();

            static FN: AtomicPtr<()> = AtomicPtr::new(detect as FnRaw);

            fn detect($($arg: $arg_ty),*) $(-> $ret_ty)? {
                #[inline(always)]
                fn select() -> FnRaw {
                    if is_x86_feature_detected!("avx2") {
                        #[cfg(feature = "unstable")]
                        {
                            if is_x86_feature_detected!("avx512vl") && is_x86_feature_detected!("avx512bw") {
                                return super::evex::$ty::$name as FnRaw;
                            }
                        }
                        super::avx2::$ty::$name as FnRaw
                    } else {
                        // SSE2 is supported for all for x86_64 processors.
                        super::sse2::$ty::$name as FnRaw
                    }
                }
                let f = select();

                FN.store(f, Ordering::Relaxed);

                // SAFETY: By virtue of the caller contract, $fn_ty is a function
                //         pointer, which is always safe to transmute with `*mut ()`.
                //         Also, if `f` is the AVX2 routine, then it is guaranteed to be
                //         supported since we checked the `avx2` feature.
                unsafe {
                    (mem::transmute::<FnRaw, $fn_ty>(f))($($arg),*)
                }
            }

            #[inline(always)]
            pub unsafe fn $name($($arg: $arg_ty),*) $(-> $ret_ty)? {
                // SAFETY: By virtue of the caller contract, $fn_ty is a function
                //         pointer, which is always safe to transmute with `*mut ()`.
                //         Also, if `f` is the AVX2 routine, then it is guaranteed to be
                //         supported since we checked the `avx2` feature.
                let f = FN.load(Ordering::Relaxed);
                (mem::transmute::<FnRaw, $fn_ty>(f))($($arg),*)
            }
        }
    }
}

unsafe_ifuncs! {
    fn i16::wmemchr(needle: i16, haystack: *const i16, len: usize) -> Option<usize>;
    fn i32::wmemchr(needle: i32, haystack: *const i32, len: usize) -> Option<usize>;
}

pub(crate) struct Kernel;

impl KernelFn<u16> for Kernel {
    fn kernel(needle: u16, haystack: &[u16]) -> Option<usize> {
        unsafe {
            i16::wmemchr(
                needle as i16,
                haystack.as_ptr() as *const i16,
                haystack.len(),
            )
        }
    }
}

impl KernelFn<i16> for Kernel {
    fn kernel(needle: i16, haystack: &[i16]) -> Option<usize> {
        unsafe { i16::wmemchr(needle, haystack.as_ptr(), haystack.len()) }
    }
}

impl KernelFn<u32> for Kernel {
    fn kernel(needle: u32, haystack: &[u32]) -> Option<usize> {
        unsafe {
            i32::wmemchr(
                needle as i32,
                haystack.as_ptr() as *const i32,
                haystack.len(),
            )
        }
    }
}

impl KernelFn<i32> for Kernel {
    fn kernel(needle: i32, haystack: &[i32]) -> Option<usize> {
        unsafe { i32::wmemchr(needle, haystack.as_ptr(), haystack.len()) }
    }
}
