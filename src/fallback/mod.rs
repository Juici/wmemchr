//! Pure Rust platform independent implementation designed for speed.

use core::mem;

use crate::char::{KernelFn, Wide};

mod packed;

use self::packed::{simd_eq, NonZeroPacked, Pack, Packed};

/// Returns the index of the first occurrence of a wide character in a slice,
/// or [`None`] if the character is not found.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use wchar::wch;
/// use wmemchr::fallback::wmemchr;
///
/// let haystack = wch!(u16, "foo bar");
///
/// assert_eq!(wmemchr(wch!(u16, 'o'), haystack), Some(1));
/// assert_eq!(wmemchr(wch!(u16, 'z'), haystack), None);
/// ```
#[inline(always)]
pub fn wmemchr<T: Wide>(needle: T, haystack: &[T]) -> Option<usize> {
    T::wmemchr_fallback(needle, haystack)
}

pub(crate) struct Kernel;

impl<T: Pack> KernelFn<T> for Kernel {
    fn kernel(needle: T, haystack: &[T]) -> Option<usize> {
        const VECTOR_SIZE: usize = mem::size_of::<Packed>();
        const VECTOR_ALIGN: usize = VECTOR_SIZE - 1;

        const LOOP_SIZE: usize = 4 * VECTOR_SIZE;
        let loop_elements = 4 * T::LANES;

        let start = haystack.as_ptr();
        let mut ptr = start;

        unsafe {
            let end = start.add(haystack.len());

            debug_assert!(start <= end);

            // If haystack length is less than number of elements in a packed vector,
            // then do a simple forward search.
            if haystack.len() < T::LANES {
                while ptr < end {
                    if *ptr == needle {
                        return Some(ptr.offset_from(start) as usize);
                    }
                    ptr = ptr.add(1);
                }
                return None;
            }

            debug_assert!(end.offset_from(start) as usize >= T::LANES);

            // Broadcast the needle across the elements of the vector.
            let v_needle = needle.broadcast();

            if let Some(pos) = forward_search_unaligned(start, end, ptr, v_needle) {
                return Some(pos);
            }

            // Align `ptr` to improve read performance in loop.
            // This calculation is based on byte pointer, and not the scaled addition.
            ptr = {
                let align_offset = VECTOR_SIZE - ((start as usize) & VECTOR_ALIGN);
                (start as *const u8).add(align_offset) as *const T
            };

            // The pointer will advance at least one element and at most by the
            // number of elements in one vector.
            debug_assert!(start < ptr);
            debug_assert!(ptr.offset_from(start) as usize <= T::LANES);

            if let Some(loop_end) = (end as usize).checked_sub(LOOP_SIZE) {
                while (ptr as usize) <= loop_end {
                    debug_assert_eq!((ptr as usize) % VECTOR_SIZE, 0);

                    let p = ptr as *const Packed;

                    // Load 4 vectors of characters.
                    let a = *p;
                    let b = *p.add(1);
                    let c = *p.add(2);
                    let d = *p.add(3);

                    // Look for needle in vectors.
                    let eq_a = simd_eq::<T>(a, v_needle);
                    let eq_b = simd_eq::<T>(b, v_needle);
                    let eq_c = simd_eq::<T>(c, v_needle);
                    let eq_d = simd_eq::<T>(d, v_needle);

                    // Determine if any vectors contained the needle.
                    let or_ab = eq_a | eq_b;
                    let or_cd = eq_c | eq_d;
                    let or = or_ab | or_cd;

                    // If any vector contains the needle, we will search for it in each vector.
                    if or != 0 {
                        // Keep track of the offset from the start of the haystack.
                        let mut offset = ptr.offset_from(start) as usize;

                        if let Some(mask) = NonZeroPacked::new(eq_a) {
                            return Some(offset + forward_pos::<T>(mask));
                        }
                        offset += T::LANES;

                        if let Some(mask) = NonZeroPacked::new(eq_b) {
                            return Some(offset + forward_pos::<T>(mask));
                        }
                        offset += T::LANES;

                        if let Some(mask) = NonZeroPacked::new(eq_c) {
                            return Some(offset + forward_pos::<T>(mask));
                        }
                        offset += T::LANES;

                        debug_assert_ne!(eq_d, 0);
                        let mask = NonZeroPacked::new_unchecked(eq_d);
                        return Some(offset + forward_pos::<T>(mask));
                    }

                    ptr = ptr.add(loop_elements);
                }
            }

            if let Some(loop_end) = (end as usize).checked_sub(VECTOR_SIZE) {
                while (ptr as usize) <= loop_end {
                    debug_assert_eq!((ptr as usize) % VECTOR_SIZE, 0);

                    let chunk = *(ptr as *const Packed);
                    let eq = simd_eq::<T>(chunk, v_needle);

                    if let Some(mask) = NonZeroPacked::new(eq) {
                        let offset = ptr.offset_from(start) as usize;
                        return Some(offset + forward_pos::<T>(mask));
                    }

                    ptr = ptr.add(T::LANES);
                }
            }

            // Invariant: `0 <= end - ptr < T::LANES`.

            // We can search the remaining elements by shifting `ptr` back and doing an
            // unaligned forward search.

            if ptr < end {
                let remaining = end.offset_from(ptr) as usize;

                debug_assert!(remaining < T::LANES);
                ptr = ptr.sub(T::LANES - remaining);
                debug_assert_eq!(end.offset_from(ptr) as usize, T::LANES);

                return forward_search_unaligned(start, end, ptr, v_needle);
            }

            None
        }
    }
}

#[inline]
unsafe fn forward_search_unaligned<T: Pack>(
    start: *const T,
    end: *const T,
    ptr: *const T,
    v_needle: Packed,
) -> Option<usize> {
    debug_assert!(start <= ptr);
    debug_assert!(end.offset_from(ptr) as usize >= T::LANES);

    let chunk = (ptr as *const Packed).read_unaligned();
    let eq = simd_eq::<T>(chunk, v_needle);

    if let Some(mask) = NonZeroPacked::new(eq) {
        let offset = ptr.offset_from(start) as usize;
        Some(offset + forward_pos::<T>(mask))
    } else {
        None
    }
}

#[inline(always)]
fn forward_pos<T: Pack>(mask: NonZeroPacked) -> usize {
    (bsf!(mask) as usize) / T::BITS
}
