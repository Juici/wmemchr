use crate::Wide;

pub fn wmemchr<T: Wide>(needle: T, haystack: &[T]) -> Option<usize> {
    haystack.iter().position(|&c| c == needle)
}
