use core::mem;

use packed_simd::SimdVector;

use crate::char::Wide;
use crate::fallback::forward_search;

pub trait SimdWide: Wide {
    type Packed: SimdVector<Element = Self> + Copy;

    fn splat(value: Self) -> Self::Packed;
    fn packed_contains(chunk: Self::Packed, packed_value: Self::Packed) -> bool;
}

macro_rules! impl_simd {
    ($ty:ty, $packed:ty) => {
        impl SimdWide for $ty {
            type Packed = $packed;

            #[inline(always)]
            fn splat(value: $ty) -> $packed {
                <$packed>::splat(value)
            }

            #[inline(always)]
            fn packed_contains(chunk: $packed, packed_value: $packed) -> bool {
                chunk.eq(packed_value).any()
            }
        }
    };
}
impl_simd!(u16, packed_simd::u16x32);
impl_simd!(u32, packed_simd::u32x16);
impl_simd!(i16, packed_simd::i16x32);
impl_simd!(i32, packed_simd::i32x16);

pub fn wmemchr<T: SimdWide>(needle: T, haystack: &[T]) -> Option<usize> {
    let confirm = |n| n == needle;

    let v_needle = T::splat(needle);

    let start = haystack.as_ptr();
    let mut ptr = start;

    unsafe {
        let end = start.add(haystack.len());
        if haystack.len() < T::Packed::LANES {
            return forward_search(start, end, ptr, confirm);
        }

        let chunk = (ptr as *const T::Packed).read_unaligned();
        if T::packed_contains(chunk, v_needle) {
            return forward_search(start, end, ptr, confirm);
        }

        ptr = ptr.add(ptr.align_offset(mem::align_of::<T::Packed>()));

        debug_assert!(ptr > start);
        debug_assert!(end.sub(T::Packed::LANES) >= start);

        while ptr < end.sub(T::Packed::LANES) {
            debug_assert_eq!(0, (ptr as usize) % T::Packed::LANES);

            let chunk = *(ptr as *const T::Packed);
            if T::packed_contains(chunk, v_needle) {
                break;
            }

            ptr = ptr.add(T::Packed::LANES);
        }

        forward_search(start, end, ptr, confirm)
    }
}
