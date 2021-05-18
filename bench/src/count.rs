use wmemchr::fallback;
use wmemchr::naive;
use wmemchr::Wide;

pub fn fallback<T: Wide>(needle: T, haystack: &[T]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = fallback::wmemchr(needle, &haystack[start..]) {
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
