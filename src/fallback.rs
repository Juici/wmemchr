use crate::char::{Packed, PackedWide, Wide};

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

pub fn wmemchr<T: Wide>(needle: T, haystack: &[T]) -> Option<usize> {
    let confirm = |n| n == needle;

    let v_needle = needle.broadcast();

    let align = T::LANES - 1;

    let start = haystack.as_ptr();
    let mut ptr = start;

    unsafe {
        let end = start.add(haystack.len());
        if haystack.len() < T::LANES {
            return forward_search(start, end, ptr, confirm);
        }

        let chunk = (ptr as *const Packed).read_unaligned();
        if (chunk ^ v_needle).contains_zero::<T>() {
            return forward_search(start, end, ptr, confirm);
        }

        ptr = ptr.add(T::LANES - (start as usize & align));

        debug_assert!(ptr > start);
        debug_assert!(end.sub(T::LANES) >= start);

        while ptr < end.sub(T::LANES) {
            debug_assert_eq!(0, (ptr as usize) % T::LANES);

            let v = *(ptr as *const Packed);
            if (v ^ v_needle).contains_zero::<T>() {
                break;
            }

            ptr = ptr.add(T::LANES);
        }

        forward_search(start, end, ptr, confirm)
    }
}
