use wmemchr::fallback;
use wmemchr::naive;
use wmemchr::simd::{self, SimdWide};
use wmemchr::Wide;

pub fn simd<T: SimdWide>(needle: T, haystack: &[T]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = simd::wmemchr(needle, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}

pub fn fallback_packed<T: Wide>(needle: T, haystack: &[T]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = fallback::wmemchr_packed(needle, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}

pub fn fallback_unrolled<T: Wide>(needle: T, haystack: &[T]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = fallback::wmemchr_unrolled(needle, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}

pub fn fallback_unrolled2<T: Wide>(needle: T, haystack: &[T]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = fallback::wmemchr_unrolled2(needle, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}

pub fn naive<T: Wide>(needle: T, haystack: &[T]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = naive::wmemchr(needle, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}
