mod arch {
    #[cfg(target_arch = "x86")]
    pub use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    pub use core::arch::x86_64::*;
}

mod avx2;
mod sse2;

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
                let f = if is_x86_feature_detected!("avx2") {
                    super::avx2::$ty::$name as FnRaw
                } else {
                    // SSE2 support is assumed for x86 processors.
                    super::sse2::$ty::$name as FnRaw
                };

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
                unsafe {
                    let f = FN.load(Ordering::Relaxed);
                    (mem::transmute::<FnRaw, $fn_ty>(f))($($arg),*)
                }
            }
        }
    }
}

unsafe_ifuncs! {
    fn i16::wmemchr(needle: i16, haystack: *const i16, len: usize) -> Option<usize>;
}
