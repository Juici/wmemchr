use core::arch::x86_64::*;
use core::mem;
use core::num::{NonZeroU16, NonZeroU8};

const VECTOR_SIZE: usize = mem::size_of::<__m256i>() / mem::size_of::<i16>();
const VECTOR_ALIGN: usize = VECTOR_SIZE - 1;

const LOOP_SIZE: usize = 4 * VECTOR_SIZE;

const SMALL_VECTOR_SIZE: usize = mem::size_of::<__m128i>() / mem::size_of::<i16>();
const SMALL_VECTOR_ALIGN: usize = SMALL_VECTOR_SIZE - 1;

// Use a macro instead of a function, since the mask type can vary.
macro_rules! forward_pos {
    ($mask:expr) => {
        bsf!($mask) as usize
    };
}

#[target_feature(enable = "avx512vl,avx512bw")]
pub unsafe fn wmemchr(needle: i16, haystack: *const i16, len: usize) -> Option<usize> {
    let start = haystack;
    let end = haystack.add(len);

    debug_assert!(start <= end);

    // If haystack length is less than number of elements in a packed vector,
    // then try with a smaller vector.
    if len < VECTOR_SIZE {
        return wmemchr_small(needle, start, end, len);
    }

    debug_assert!(start <= end.sub(VECTOR_SIZE));

    let mut ptr = start;

    // Broadcast the needle across the elements of the vector.
    let v_needle = _mm256_set1_epi16(needle);

    if let Some(pos) = forward_search_unaligned(start, end, ptr, v_needle) {
        return Some(pos);
    }

    // Align `ptr` to improve read performance in loop.
    // This calculation is based on byte pointer, and not the scaled addition.
    ptr = (ptr as *const u8).add(VECTOR_SIZE - ((start as usize) & VECTOR_ALIGN)) as *const i16;

    debug_assert!(start < ptr);

    // 128 byte (64 element) loop.
    while ptr <= end.sub(LOOP_SIZE) {
        debug_assert_eq!((ptr as usize) % VECTOR_SIZE, 0);

        let p = ptr as *const __m256i;

        // Load 2 vectors of characters.
        let a = _mm256_load_si256(p);
        let b = _mm256_load_si256(p.add(1));
        let c = _mm256_load_si256(p.add(2));
        let d = _mm256_load_si256(p.add(3));

        // Look for needle in vectors.
        let mask_a = _mm256_cmpeq_epi16_mask(a, v_needle);
        let mask_b = _mm256_cmpeq_epi16_mask(b, v_needle);
        let mask_c = _mm256_cmpeq_epi16_mask(c, v_needle);
        let mask_d = _mm256_cmpeq_epi16_mask(d, v_needle);

        if let Some(mask) = NonZeroU16::new(mask_a) {
            let offset = ptr.offset_from(start) as usize;
            return Some(offset + forward_pos!(mask));
        }

        if let Some(mask) = NonZeroU16::new(mask_b) {
            let offset = ptr.offset_from(start) as usize;
            return Some(offset + VECTOR_SIZE + forward_pos!(mask));
        }

        if let Some(mask) = NonZeroU16::new(mask_c) {
            let offset = ptr.offset_from(start) as usize;
            return Some(offset + (VECTOR_SIZE * 2) + forward_pos!(mask));
        }

        if let Some(mask) = NonZeroU16::new(mask_d) {
            let offset = ptr.offset_from(start) as usize;
            return Some(offset + (VECTOR_SIZE * 3) + forward_pos!(mask));
        }

        ptr = ptr.add(LOOP_SIZE);
    }

    // 32 byte (16 element) loop.
    while ptr <= end.sub(VECTOR_SIZE) {
        debug_assert_eq!((ptr as usize) % VECTOR_SIZE, 0);

        let chunk = _mm256_load_si256(ptr as *const __m256i);
        let mask = _mm256_cmpeq_epi16_mask(chunk, v_needle);

        if let Some(mask) = NonZeroU16::new(mask) {
            let offset = ptr.offset_from(start) as usize;
            return Some(offset + forward_pos!(mask));
        }

        ptr = ptr.add(VECTOR_SIZE);
    }

    // Invariant: `0 <= end - ptr < VECTOR_SIZE`.

    // We can search the remaining elements by shifting `ptr` back and doing an
    // unaligned forward search.

    if ptr < end {
        let remaining = end.offset_from(start) as usize;

        debug_assert!(remaining < VECTOR_SIZE);
        ptr = ptr.sub(VECTOR_SIZE - remaining);
        debug_assert_eq!(end.offset_from(start) as usize, VECTOR_SIZE);

        return forward_search_unaligned(start, end, ptr, v_needle);
    }

    None
}

#[inline]
#[target_feature(enable = "avx512vl,avx512bw")]
unsafe fn forward_search_unaligned(
    start: *const i16,
    end: *const i16,
    ptr: *const i16,
    v_needle: __m256i,
) -> Option<usize> {
    debug_assert!(start <= ptr);
    debug_assert!(ptr <= end.sub(VECTOR_SIZE));

    let chunk = _mm256_loadu_epi16(ptr);
    let mask = _mm256_cmpeq_epi16_mask(chunk, v_needle);

    if let Some(mask) = NonZeroU16::new(mask) {
        let offset = ptr.offset_from(start) as usize;
        Some(offset + forward_pos!(mask))
    } else {
        None
    }
}

#[inline]
#[target_feature(enable = "avx512vl,avx512bw")]
unsafe fn wmemchr_small(
    needle: i16,
    start: *const i16,
    end: *const i16,
    len: usize,
) -> Option<usize> {
    let mut ptr = start;

    // If haystack length is less than the number of elements in a smaller
    // packed vector, then just fallback to by element search.
    if len < SMALL_VECTOR_SIZE {
        while ptr < end {
            if *ptr == needle {
                return Some(ptr.offset_from(start) as usize);
            }
            ptr = ptr.add(1);
        }
        return None;
    }

    debug_assert!(start <= end.sub(SMALL_VECTOR_SIZE));

    // Broadcast the needle across the elements of the vector.
    let v_needle = _mm_set1_epi16(needle);

    // Search the first small vector
    if let Some(pos) = forward_search_unaligned_small(start, end, ptr, v_needle) {
        return Some(pos);
    }
    ptr = ptr.add(SMALL_VECTOR_SIZE);

    // Invariant: `0 <= end - ptr < VECTOR_SIZE`.

    // We can search the remaining elements by shifting `ptr` back and doing an
    // unaligned forward search.

    if ptr < end {
        let remaining = len - SMALL_VECTOR_SIZE;

        debug_assert!(remaining < SMALL_VECTOR_SIZE);
        ptr = ptr.sub(SMALL_VECTOR_SIZE - remaining);
        debug_assert_eq!(end.offset_from(ptr) as usize, SMALL_VECTOR_SIZE);

        return forward_search_unaligned_small(start, end, ptr, v_needle);
    }

    None
}

#[inline]
#[target_feature(enable = "avx512vl,avx512bw")]
unsafe fn forward_search_unaligned_small(
    start: *const i16,
    end: *const i16,
    ptr: *const i16,
    v_needle: __m128i,
) -> Option<usize> {
    debug_assert!(start <= ptr);
    debug_assert!(ptr <= end.sub(SMALL_VECTOR_SIZE));

    let chunk = _mm_loadu_epi16(ptr);
    let mask = _mm_cmpeq_epi16_mask(chunk, v_needle);

    if let Some(mask) = NonZeroU8::new(mask) {
        let offset = ptr.offset_from(start) as usize;
        Some(offset + forward_pos!(mask))
    } else {
        None
    }
}
