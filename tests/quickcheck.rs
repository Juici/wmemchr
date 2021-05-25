#![cfg(not(miri))]

macro_rules! tests {
    ($($ty:ident),*) => {
        $(
            mod $ty {
                use quickcheck::quickcheck;
                use wmemchr::fallback;
                use wmemchr::naive;

                #[cfg(target_arch = "x86_64")]
                use wmemchr::x86_64;

                quickcheck! {
                    fn fallback(needle: $ty, haystack: Vec<$ty>) -> bool {
                        fallback::wmemchr(needle, &haystack) == naive::wmemchr(needle, &haystack)
                    }
                }

                #[cfg(target_arch = "x86_64")]
                quickcheck! {
                    fn x86_64(needle: $ty, haystack: Vec<$ty>) -> bool {
                        x86_64::wmemchr(needle, &haystack) == naive::wmemchr(needle, &haystack)
                    }
                }
            }
        )*
    };
}
tests! { u16, u32, i16, i32 }
