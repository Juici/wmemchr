use core::arch::x86_64::*;
use core::mem;
use core::num::NonZeroU8;

const VECTOR_SIZE: usize = mem::size_of::<__m256i>();
const VECTOR_ALIGN: usize = VECTOR_SIZE - 1;

const VECTOR_ELEMENTS: usize = VECTOR_SIZE / mem::size_of::<i32>();

const LOOP_SIZE: usize = 4 * VECTOR_SIZE;
const LOOP_ELEMENTS: usize = 4 * VECTOR_ELEMENTS;

const SMALL_VECTOR_ELEMENTS: usize = mem::size_of::<__m128i>() / mem::size_of::<i32>();

// Use a macro instead of a function, since the mask type can vary.
macro_rules! forward_pos {
    ($mask:expr) => {
        bsf!($mask) as usize
    };
}

#[target_feature(enable = "avx512vl,avx512bw")]
pub unsafe fn wmemchr(needle: i32, haystack: *const i32, len: usize) -> Option<usize> {
    let start = haystack;
    let end = haystack.add(len);

    debug_assert!(start <= end);

    // If haystack length is less than number of elements in a packed vector,
    // then try with a smaller vector.
    if len < VECTOR_ELEMENTS {
        return wmemchr_small(needle, start, end, len);
    }

    debug_assert!(end.offset_from(start) as usize >= VECTOR_ELEMENTS);

    let mut ptr = start;

    // Broadcast the needle across the elements of the vector.
    let v_needle = _mm256_set1_epi32(needle);

    if let Some(pos) = forward_search_unaligned(start, end, ptr, v_needle) {
        return Some(pos);
    }

    // Align `ptr` to improve read performance in loop.
    // This calculation is based on byte pointer, and not the scaled addition.
    ptr = {
        let align_offset = VECTOR_SIZE - ((start as usize) & VECTOR_ALIGN);
        (start as *const u8).add(align_offset) as *const i32
    };

    // The pointer will advance at least one element and at most by the
    // number of elements in one vector.
    debug_assert!(start < ptr);
    debug_assert!(ptr.offset_from(start) as usize <= VECTOR_ELEMENTS);

    // 128 byte (32 element) loop.
    if let Some(loop_end) = (end as usize).checked_sub(LOOP_SIZE) {
        while (ptr as usize) <= loop_end {
            debug_assert_eq!((ptr as usize) % VECTOR_SIZE, 0);

            // Load 4 vectors of characters.
            let a = _mm256_load_epi32(ptr);
            let b = _mm256_load_epi32(ptr.add(VECTOR_SIZE));
            let c = _mm256_load_epi32(ptr.add(2 * VECTOR_SIZE));
            let d = _mm256_load_epi32(ptr.add(3 * VECTOR_SIZE));

            // Look for needle in vectors.
            let mask_a = _mm256_cmpeq_epi32_mask(a, v_needle);
            let mask_b = _mm256_cmpeq_epi32_mask(b, v_needle);
            let mask_c = _mm256_cmpeq_epi32_mask(c, v_needle);
            let mask_d = _mm256_cmpeq_epi32_mask(d, v_needle);

            if let Some(mask) = NonZeroU8::new(mask_a) {
                let offset = ptr.offset_from(start) as usize;
                return Some(offset + forward_pos!(mask));
            }

            if let Some(mask) = NonZeroU8::new(mask_b) {
                let offset = ptr.offset_from(start) as usize;
                return Some(offset + VECTOR_ELEMENTS + forward_pos!(mask));
            }

            if let Some(mask) = NonZeroU8::new(mask_c) {
                let offset = ptr.offset_from(start) as usize;
                return Some(offset + (VECTOR_ELEMENTS * 2) + forward_pos!(mask));
            }

            if let Some(mask) = NonZeroU8::new(mask_d) {
                let offset = ptr.offset_from(start) as usize;
                return Some(offset + (VECTOR_ELEMENTS * 3) + forward_pos!(mask));
            }

            ptr = ptr.add(LOOP_ELEMENTS);
        }
    }

    // 32 byte (8 element) loop.
    if let Some(loop_end) = (end as usize).checked_sub(VECTOR_SIZE) {
        while (ptr as usize) <= loop_end {
            debug_assert_eq!((ptr as usize) % VECTOR_SIZE, 0);

            let chunk = _mm256_load_epi32(ptr);
            let mask = _mm256_cmpeq_epi32_mask(chunk, v_needle);

            if let Some(mask) = NonZeroU8::new(mask) {
                let offset = ptr.offset_from(start) as usize;
                return Some(offset + forward_pos!(mask));
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
#[target_feature(enable = "avx512vl,avx512bw")]
unsafe fn forward_search_unaligned(
    start: *const i32,
    end: *const i32,
    ptr: *const i32,
    v_needle: __m256i,
) -> Option<usize> {
    debug_assert!(start <= ptr);
    debug_assert!(end.offset_from(ptr) as usize >= VECTOR_ELEMENTS);

    let chunk = _mm256_loadu_epi32(ptr);
    let mask = _mm256_cmpeq_epi32_mask(chunk, v_needle);

    if let Some(mask) = NonZeroU8::new(mask) {
        let offset = ptr.offset_from(start) as usize;
        Some(offset + forward_pos!(mask))
    } else {
        None
    }
}

#[inline]
#[target_feature(enable = "avx512vl,avx512bw")]
unsafe fn wmemchr_small(
    needle: i32,
    start: *const i32,
    end: *const i32,
    len: usize,
) -> Option<usize> {
    let mut ptr = start;

    // If haystack length is less than the number of elements in a smaller
    // packed vector, then just fallback to by element search.
    if len < SMALL_VECTOR_ELEMENTS {
        while ptr < end {
            if *ptr == needle {
                return Some(ptr.offset_from(start) as usize);
            }
            ptr = ptr.add(1);
        }
        return None;
    }

    debug_assert!(end.offset_from(start) as usize >= SMALL_VECTOR_ELEMENTS);

    // Broadcast the needle across the elements of the vector.
    let v_needle = _mm_set1_epi32(needle);

    // Search the first small vector
    if let Some(pos) = forward_search_unaligned_small(start, end, ptr, v_needle) {
        return Some(pos);
    }
    ptr = ptr.add(SMALL_VECTOR_ELEMENTS);

    // Invariant: `0 <= end - ptr < SMALL_VECTOR_SIZE`.

    // We can search the remaining elements by shifting `ptr` back and doing an
    // unaligned forward search.

    if ptr < end {
        let remaining = len - SMALL_VECTOR_ELEMENTS;

        debug_assert!(remaining < SMALL_VECTOR_ELEMENTS);
        ptr = ptr.sub(SMALL_VECTOR_ELEMENTS - remaining);
        debug_assert_eq!(end.offset_from(ptr) as usize, SMALL_VECTOR_ELEMENTS);

        return forward_search_unaligned_small(start, end, ptr, v_needle);
    }

    None
}

#[inline]
#[target_feature(enable = "avx512vl,avx512bw")]
unsafe fn forward_search_unaligned_small(
    start: *const i32,
    end: *const i32,
    ptr: *const i32,
    v_needle: __m128i,
) -> Option<usize> {
    debug_assert!(start <= ptr);
    debug_assert!(end.offset_from(ptr) as usize >= SMALL_VECTOR_ELEMENTS);

    let chunk = _mm_loadu_epi32(ptr);
    let mask = _mm_cmpeq_epi32_mask(chunk, v_needle);

    if let Some(mask) = NonZeroU8::new(mask) {
        let offset = ptr.offset_from(start) as usize;
        Some(offset + forward_pos!(mask))
    } else {
        None
    }
}
