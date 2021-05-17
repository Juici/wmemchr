use core::cmp;
use core::ptr;

use crate::char::{UnsignedWide, Wide};

#[inline(always)]
fn contains_zero_wchar<T: Wide>(x: usize) -> bool {
    (x.wrapping_sub(<T::Unsigned>::LO_USIZE) & !x & <T::Unsigned>::HI_USIZE) != 0
}

#[inline(always)]
pub(crate) unsafe fn forward_search<T: Wide, F: Fn(T) -> bool>(
    start: *const T,
    end: *const T,
    mut ptr: *const T,
    confirm: F,
) -> Option<usize> {
    debug_assert!(start <= ptr);
    debug_assert!(ptr <= end);

    while ptr < end {
        if confirm(*ptr) {
            return Some(ptr.offset_from(start) as usize);
        }
        ptr = ptr.add(1);
    }

    None
}

pub fn wmemchr_packed<T: Wide>(needle: T, haystack: &[T]) -> Option<usize> {
    let v_needle = needle.unsigned().broadcast_usize();
    let confirm = |n| n == needle;
    let loop_size = cmp::min(T::LOOP_SIZE, haystack.len());
    let align = T::USIZE_WIDES - 1;
    let start = haystack.as_ptr();
    let mut ptr = start;

    unsafe {
        let end = start.add(haystack.len());
        if haystack.len() < T::USIZE_WIDES {
            return forward_search(start, end, ptr, confirm);
        }

        let chunk = (ptr as *const usize).read_unaligned();
        if contains_zero_wchar::<T>(chunk ^ v_needle) {
            return forward_search(start, end, ptr, confirm);
        }

        ptr = ptr.add(T::USIZE_WIDES - (start as usize & align));

        debug_assert!(ptr > start);
        debug_assert!(end.sub(T::USIZE_WIDES) >= start);

        while loop_size == T::LOOP_SIZE && ptr < end.sub(loop_size) {
            debug_assert_eq!(0, (ptr as usize) % T::USIZE_WIDES);

            let a = *(ptr as *const usize);
            let b = *(ptr.add(T::USIZE_WIDES) as *const usize);

            let eq_a = contains_zero_wchar::<T>(a ^ v_needle);
            let eq_b = contains_zero_wchar::<T>(b ^ v_needle);
            if eq_a || eq_b {
                break;
            }

            ptr = ptr.add(T::LOOP_SIZE);
        }

        forward_search(start, end, ptr, confirm)
    }
}

pub fn wmemchr_unrolled<T: Wide>(needle: T, haystack: &[T]) -> Option<usize> {
    let mut i = 0;
    let n = haystack.len();

    macro_rules! check_needle {
        ($pos:expr) => {{
            let pos = $pos;
            if unsafe { *haystack.get_unchecked(pos) } == needle {
                return Some(pos);
            }
        }};
    }

    // Unroll loop 4 times.
    while i + 4 <= n {
        check_needle!(i);
        check_needle!(i + 1);
        check_needle!(i + 2);
        check_needle!(i + 3);
        i += 4;
    }

    // Up to 3 remaining elements.
    // 1st remaining element.
    if i < n {
        check_needle!(i);
        i += 1;
    }
    // 2nd remaining element.
    if i < n {
        check_needle!(i);
        i += 1;
    }
    // 3rd remaining element.
    if i < n {
        check_needle!(i);
    }

    None
}

pub fn wmemchr_unrolled2<T: Wide>(needle: T, haystack: &[T]) -> Option<usize> {
    unsafe fn imp<T: Wide>(mut s: *const T, c: T, mut n: usize) -> *const T {
        // Unroll loop 4 times.
        while n >= 4 {
            if *s == c {
                return s;
            }
            if *s.add(1) == c {
                return s.add(1);
            }
            if *s.add(2) == c {
                return s.add(2);
            }
            if *s.add(3) == c {
                return s.add(3);
            }

            s = s.add(4);
            n -= 4;
        }

        // Up to 3 remaining elements.
        // 1st remaining element.
        if n > 0 {
            if *s == c {
                return s;
            }
            s = s.add(1);
            n -= 1;
        }
        // 2nd remaining element.
        if n > 0 {
            if *s == c {
                return s;
            }
            s = s.add(1);
            n -= 1;
        }
        // 3rd remaining element.
        if n > 0 {
            if *s == c {
                return s;
            }
        }

        ptr::null()
    }

    let p = unsafe { imp(haystack.as_ptr(), needle, haystack.len()) };
    if p.is_null() {
        None
    } else {
        Some(unsafe { p.offset_from(haystack.as_ptr()) as usize })
    }
}
