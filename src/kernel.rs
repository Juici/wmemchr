use crate::Wide;

pub trait Kernel {
    type Element: Wide;

    fn kernel(needle: Self::Element, haystack: &[Self::Element]) -> Option<usize>;
}

/// Kernel selector.
pub trait KernelSelect<T> {
    fn select<K>(self, kernel: K)
    where
        K: Kernel<Element = T>,
        T: Wide;
}
