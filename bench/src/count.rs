use wmemchr::fallback;
use wmemchr::naive;
#[cfg(target_arch = "x86_64")]
use wmemchr::x86_64;
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

#[cfg(target_arch = "x86_64")]
pub fn x86_64<T: Wide>(needle: T, haystack: &[T]) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(i) = x86_64::wmemchr(needle, &haystack[start..]) {
        count += 1;
        start += i + 1;
    }
    count
}
