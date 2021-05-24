//! Simple tests that can be run under Miri.

macro_rules! tests {
    ($($ty:ident),*) => {
        $(
            mod $ty {
                use wchar::wch;

                use crate::wmemchr;

                tests! { @ascii $ty }
                tests! { @complex $ty }
                tests! { @emoji $ty }
            }
        )*
    };
    (@ascii $ty:ident) => {
        #[test]
        fn ascii() {
            let haystack: &[$ty] = wch!($ty, "abcda");

            let needle: $ty = wch!($ty, 'a');
            assert_eq!(wmemchr(needle, haystack), Some(0));

            let needle: $ty = wch!($ty, 'c');
            assert_eq!(wmemchr(needle, haystack), Some(2));

            let needle: $ty = wch!($ty, 'z');
            assert_eq!(wmemchr(needle, haystack), None);
        }
    };
    (@complex $ty:ident) => {
        #[test]
        fn complex() {
            let haystack: &[$ty] = wch!($ty, "Löwe 老虎 Léopard Gepardi");

            let needle: $ty = wch!($ty, 'ö');
            assert_eq!(wmemchr(needle, haystack), Some(1));

            let needle: $ty = wch!($ty, '虎');
            assert_eq!(wmemchr(needle, haystack), Some(6));

            let needle: $ty = wch!($ty, 'é');
            assert_eq!(wmemchr(needle, haystack), Some(9));

            let needle: $ty = wch!($ty, 'o');
            assert_eq!(wmemchr(needle, haystack), Some(10));

            let needle: $ty = wch!($ty, '京');
            assert_eq!(wmemchr(needle, haystack), None);
        }

        #[test]
        fn rtl() {
            let haystack: &[$ty] = wch!($ty, "حل كيمياء");

            let needle: $ty = wch!($ty, 'ح');
            assert_eq!(wmemchr(needle, haystack), Some(0));

            let needle: $ty = wch!($ty, 'ك');
            assert_eq!(wmemchr(needle, haystack), Some(3));

            let needle: $ty = wch!($ty, '٣');
            assert_eq!(wmemchr(needle, haystack), None);
        }
    };
    (@emoji u16) => {};
    (@emoji i16) => {};
    (@emoji $ty:ident) => {
        #[test]
        fn emoji() {
            let haystack: &[$ty] = wch!($ty, "🦀💖🧡💚💙");

            let needle: $ty = wch!($ty, '🦀');
            assert_eq!(wmemchr(needle, haystack), Some(0));

            let needle: $ty = wch!($ty, '💖');
            assert_eq!(wmemchr(needle, haystack), Some(1));

            let needle: $ty = wch!($ty, '💚');
            assert_eq!(wmemchr(needle, haystack), Some(3));

            let needle: $ty = wch!($ty, '💜');
            assert_eq!(wmemchr(needle, haystack), None);

            let needle: $ty = wch!($ty, '💝');
            assert_eq!(wmemchr(needle, haystack), None);
        }
    };
}
tests! { u16, u32, i16, i32 }
