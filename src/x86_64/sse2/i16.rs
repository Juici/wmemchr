use core::arch::x86_64::*;
use core::mem;
use core::num::NonZeroI32;

const VECTOR_SIZE: usize = mem::size_of::<__m128i>();
const VECTOR_ALIGN: usize = VECTOR_SIZE - 1;

const VECTOR_ELEMENTS: usize = VECTOR_SIZE / mem::size_of::<i16>();

const LOOP_SIZE: usize = 4 * VECTOR_SIZE;
const LOOP_ELEMENTS: usize = 4 * VECTOR_ELEMENTS;

#[target_feature(enable = "sse2")]
pub unsafe fn wmemchr(needle: i16, haystack: *const i16, len: usize) -> Option<usize> {
    let start = haystack;
    let end = haystack.add(len);
    let mut ptr = start;

    debug_assert!(start <= end);

    // If haystack length is less than number of elements in a packed vector,
    // then do a simple forward search.
    if len < VECTOR_ELEMENTS {
        while ptr < end {
            if *ptr == needle {
                return Some(ptr.offset_from(start) as usize);
            }
            ptr = ptr.add(1);
        }
        return None;
    }

    debug_assert!(end.offset_from(start) as usize >= VECTOR_ELEMENTS);

    // Broadcast the needle across the elements of the vector.
    let v_needle = _mm_set1_epi16(needle);

    if let Some(pos) = forward_search_unaligned(start, end, ptr, v_needle) {
        return Some(pos);
    }

    // Align `ptr` to improve read performance in loop.
    // This calculation is based on byte pointer, and not the scaled addition.
    ptr = {
        let align_offset = VECTOR_SIZE - ((start as usize) & VECTOR_ALIGN);
        (start as *const u8).add(align_offset) as *const i16
    };

    // The pointer will advance at least one element and at most by the
    // number of elements in one vector.
    debug_assert!(start < ptr);
    debug_assert!(ptr.offset_from(start) as usize <= VECTOR_ELEMENTS);

    // 64 byte (32 element) loop.
    if let Some(loop_end) = (end as usize).checked_sub(LOOP_SIZE) {
        while (ptr as usize) <= loop_end {
            debug_assert_eq!((ptr as usize) % VECTOR_SIZE, 0);

            let p = ptr as *const __m128i;

            // Load 4 vectors of characters.
            let a = _mm_load_si128(p);
            let b = _mm_load_si128(p.add(1));
            let c = _mm_load_si128(p.add(2));
            let d = _mm_load_si128(p.add(3));

            // Look for needle in vectors.
            let eq_a = _mm_cmpeq_epi16(a, v_needle);
            let eq_b = _mm_cmpeq_epi16(b, v_needle);
            let eq_c = _mm_cmpeq_epi16(c, v_needle);
            let eq_d = _mm_cmpeq_epi16(d, v_needle);

            // Determine if any vectors contained the needle.
            let or_ab = _mm_or_si128(eq_a, eq_b);
            let or_cd = _mm_or_si128(eq_c, eq_d);
            let or = _mm_or_si128(or_ab, or_cd);

            // If any vector contains the needle, we will search for it in each vector.
            if _mm_movemask_epi8(or) != 0 {
                // Keep track of the offset from the start of the haystack.
                let mut offset = ptr.offset_from(start) as usize;

                let mask = _mm_movemask_epi8(eq_a);
                if let Some(mask) = NonZeroI32::new(mask) {
                    return Some(offset + forward_pos(mask));
                }
                offset += VECTOR_ELEMENTS;

                let mask = _mm_movemask_epi8(eq_b);
                if let Some(mask) = NonZeroI32::new(mask) {
                    return Some(offset + forward_pos(mask));
                }
                offset += VECTOR_ELEMENTS;

                let mask = _mm_movemask_epi8(eq_c);
                if let Some(mask) = NonZeroI32::new(mask) {
                    return Some(offset + forward_pos(mask));
                }
                offset += VECTOR_ELEMENTS;

                let mask = _mm_movemask_epi8(eq_d);
                debug_assert_ne!(mask, 0);
                let mask = NonZeroI32::new_unchecked(mask);
                return Some(offset + forward_pos(mask));
            }

            ptr = ptr.add(LOOP_ELEMENTS);
        }
    }

    // 16 byte (8 element) loop.
    if let Some(loop_end) = (end as usize).checked_sub(VECTOR_SIZE) {
        while (ptr as usize) <= loop_end {
            debug_assert_eq!((ptr as usize) % VECTOR_SIZE, 0);

            let chunk = _mm_load_si128(ptr as *const __m128i);
            let eq = _mm_cmpeq_epi16(chunk, v_needle);

            let mask = _mm_movemask_epi8(eq);
            if let Some(mask) = NonZeroI32::new(mask) {
                let offset = ptr.offset_from(start) as usize;
                return Some(offset + forward_pos(mask));
            }

            ptr = ptr.add(VECTOR_ELEMENTS);
        }
    }

    // Invariant: `0 <= end - ptr < VECTOR_SIZE`.

    // We can search the remaining elements by shifting `ptr` back and doing an
    // unaligned forward search.

    if ptr < end {
        let remaining = end.offset_from(ptr) as usize;

        debug_assert!(remaining < VECTOR_ELEMENTS);
        ptr = ptr.sub(VECTOR_ELEMENTS - remaining);
        debug_assert_eq!(end.offset_from(ptr) as usize, VECTOR_ELEMENTS);

        return forward_search_unaligned(start, end, ptr, v_needle);
    }

    None
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn forward_search_unaligned(
    start: *const i16,
    end: *const i16,
    ptr: *const i16,
    v_needle: __m128i,
) -> Option<usize> {
    debug_assert!(start <= ptr);
    debug_assert!(end.offset_from(ptr) as usize >= VECTOR_ELEMENTS);

    let chunk = _mm_loadu_si128(ptr as *const __m128i);
    let eq = _mm_cmpeq_epi16(chunk, v_needle);

    let mask = _mm_movemask_epi8(eq);
    if let Some(mask) = NonZeroI32::new(mask) {
        let offset = ptr.offset_from(start) as usize;
        Some(offset + forward_pos(mask))
    } else {
        None
    }
}

/// Get the forward position in a mask obtained from `_mm_movemask_epi8`.
///
/// Unfortunately no `_mm_movemask_epi16` function exists, and we cannot use
/// `_mm_cmpeq_epi16_mask` on without AVX512VL + AVX512BW.
///
/// So we will make use of `_mm_movemask_epi8` and use trailing zeros to get
/// the offset of the match, then divide that by 2 to get the real position.
///
/// # Notes
///
/// We take advantage of the fact that we check the mask is non-zero, in order
/// to optimise this function.
#[inline(always)]
pub fn forward_pos(mask: NonZeroI32) -> usize {
    (bsf!(mask) as usize) >> 1
}
