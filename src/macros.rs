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
#[rustversion::since(1.53)]
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

/// Search forward for the first set bit, in a non-zero bitmask.
#[rustversion::before(1.53)]
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
