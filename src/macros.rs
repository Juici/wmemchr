/// Assert a condition at compile time.
macro_rules! static_assert {
    ($cond:expr) => {
        const _: [(); 0 - !{
            const COND: bool = $cond;
            COND
        } as usize] = [];
    };
}

cfg_if::cfg_if! {
    if #[cfg(rustc_1_53)] {
        /// Search forward for the first set bit, in a non-zero bitmask.
        macro_rules! bsf {
            ($mask:expr) => {
                match $mask {
                    #[cfg(target_endian = "little")]
                    mask => mask.trailing_zeros(),
                    #[cfg(target_endian = "big")]
                    mask => mask.leading_zeros(),
                }
            };
        }
    } else {
        /// Search forward for the first set bit, in a non-zero bitmask.
        macro_rules! bsf {
            ($mask:expr) => {
                match ($mask).get() {
                    #[allow(unsafe_unsafe)]
                    0 => unsafe { ::core::hint::unreachable_unchecked() },
                    #[cfg(target_endian = "little")]
                    mask => mask.trailing_zeros(),
                    #[cfg(target_endian = "big")]
                    mask => mask.leading_zeros(),
                }
            };
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        /// Test at runtime whether a CPU feature is available on x86/x86_64 platforms.
        macro_rules! is_x86_feature_detected {
            ($feature:tt) => {
                ::std::is_x86_feature_detected!($feature)
            };
        }
    } else if #[cfg(feature = "unstable")] {
        /// Test at runtime whether a CPU feature is available on x86/x86_64 platforms.
        macro_rules! is_x86_feature_detected {
            ($feature:tt) => {
                ::std_detect::is_x86_feature_detected!($feature)
            };
        }
    } else {
        /// Test at runtime whether a CPU feature is available on x86/x86_64 platforms.
        macro_rules! is_x86_feature_detected {
            ($feature:tt) => {
                ::core::cfg!(target_feature = $feature)
            };
        }
    }
}
