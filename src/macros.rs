#![allow(unused_macros)]

/// Assert a condition at compile time.
macro_rules! static_assert {
    ($cond:expr) => {
        const _: [(); 0 - !{
            const COND: bool = $cond;
            COND
        } as usize] = [];
    };
}

/// Search forward for the first set bit, in a non-zero bitmask.
#[cfg(rustc_1_53)]
macro_rules! bsf {
    ($mask:expr) => {
        match $mask {
            #[cfg(target_endian = "little")]
            mask => mask.trailing_zeros(),
            #[cfg(not(target_endian = "little"))]
            mask => mask.leading_zeros(),
        }
    };
}
/// Search forward for the first set bit, in a non-zero bitmask.
#[cfg(not(rustc_1_53))]
macro_rules! bsf {
    ($mask:expr) => {
        match ($mask).get() {
            #[allow(unsafe_unsafe)]
            0 => unsafe { ::core::hint::unreachable_unchecked() },
            #[cfg(target_endian = "little")]
            mask => mask.trailing_zeros(),
            #[cfg(not(target_endian = "little"))]
            mask => mask.leading_zeros(),
        }
    };
}

/// Test at runtime whether a CPU feature is available on x86/x86_64 platforms.
#[cfg(feature = "std")]
macro_rules! is_x86_feature_detected {
    ($feature:tt) => {
        ::std::is_x86_feature_detected!($feature)
    };
}
/// Test at runtime whether a CPU feature is available on x86/x86_64 platforms.
#[cfg(not(feature = "std"))]
macro_rules! is_x86_feature_detected {
    ($feature:tt) => {
        ::core::cfg!(target_feature = $feature)
    };
}
