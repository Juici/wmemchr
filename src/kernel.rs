use crate::Wide;

pub trait Kernel {
    type Element: Wide;

    fn kernel(needle: Self::Element, haystack: &[Self::Element]) -> Option<usize>;
}
