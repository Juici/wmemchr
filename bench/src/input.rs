use wchar::{include_wch, wch};
use wmemchr::Wide;

/// A description of an input to run benchmarks on.
#[derive(Clone, Copy, Debug)]
pub struct Input<T: Wide> {
    /// The wide character string to search.
    pub corpus: &'static [T],
    /// Distinct characters that never occur.
    pub never: &'static [SearchChar<T>],
    /// Distinct characters that occur very rarely (<0.1%).
    pub rare: &'static [SearchChar<T>],
    /// Distinct characters that are uncommon (~1%).
    pub uncommon: &'static [SearchChar<T>],
    /// Distinct characters that are common (~5%).
    pub common: &'static [SearchChar<T>],
    /// Distinct characters that are very common (~10%).
    pub very_common: &'static [SearchChar<T>],
    /// Distinct characters that are super common (>90%).
    pub super_common: &'static [SearchChar<T>],
}

/// A description of a search for a particular wide character.
#[derive(Clone, Copy, Debug)]
pub struct Search<T: Wide> {
    /// The wide character string to search.
    pub corpus: &'static [T],
    /// The wide character to search for.
    pub value: SearchChar<T>,
}

/// A description of a single wide character to search for.
#[derive(Clone, Copy, Debug)]
pub struct SearchChar<T: Wide> {
    /// A wide character.
    pub value: T,
    /// The number of times it is expected to occur.
    pub count: usize,
}

macro_rules! input_fns {
    ($($name:ident),*) => {
        impl<T: Wide> Input<T> {
            $(
                pub fn $name(&self) -> Option<Search<T>> {
                    if self.$name.is_empty() {
                        None
                    } else {
                        Some(Search {
                            corpus: self.corpus,
                            value: self.$name[0],
                        })
                    }
                }
            )*
        }
    };
}
input_fns!(never, rare, uncommon, common, very_common, super_common);

macro_rules! huge {
    ($(
        $vis:vis const $name:ident: Input<$ty:ident>;
    )*) => {
        $(
            $vis const $name: Input<$ty> = Input {
                corpus: include_wch!($ty, "../data/sherlock/huge.txt"),
                never: &[
                    SearchChar {
                        value: wch!($ty, '<'),
                        count: 0,
                    },
                    SearchChar {
                        value: wch!($ty, '>'),
                        count: 0,
                    },
                    SearchChar {
                        value: wch!($ty, '='),
                        count: 0,
                    },
                ],
                rare: &[
                    SearchChar {
                        value: wch!($ty, 'z'),
                        count: 151,
                    },
                    SearchChar {
                        value: wch!($ty, 'R'),
                        count: 275,
                    },
                    SearchChar {
                        value: wch!($ty, 'J'),
                        count: 120,
                    },
                ],
                uncommon: &[
                    SearchChar {
                        value: wch!($ty, 'b'),
                        count: 6124,
                    },
                    SearchChar {
                        value: wch!($ty, 'p'),
                        count: 6989,
                    },
                    SearchChar {
                        value: wch!($ty, '.'),
                        count: 6425,
                    },
                ],
                common: &[
                    SearchChar {
                        value: wch!($ty, 'a'),
                        count: 35301,
                    },
                    SearchChar {
                        value: wch!($ty, 't'),
                        count: 39268,
                    },
                    SearchChar {
                        value: wch!($ty, 'o'),
                        count: 34495,
                    },
                ],
                very_common: &[SearchChar {
                    value: wch!($ty, ' '),
                    count: 97626,
                }],
                super_common: &[],
            };
        )*
    };
}

macro_rules! small {
    ($(
        $vis:vis const $name:ident: Input<$ty:ident>;
    )*) => {
        $(
            $vis const $name: Input<$ty> = Input {
                corpus: include_wch!($ty, "../data/sherlock/small.txt"),
                never: &[
                    SearchChar {
                        value: wch!($ty, '<'),
                        count: 0,
                    },
                    SearchChar {
                        value: wch!($ty, '>'),
                        count: 0,
                    },
                    SearchChar {
                        value: wch!($ty, '='),
                        count: 0,
                    },
                ],
                rare: &[
                    SearchChar {
                        value: wch!($ty, 'R'),
                        count: 1,
                    },
                    SearchChar {
                        value: wch!($ty, 'P'),
                        count: 1,
                    },
                    SearchChar {
                        value: wch!($ty, 'T'),
                        count: 1,
                    },
                ],
                uncommon: &[
                    SearchChar {
                        value: wch!($ty, 'b'),
                        count: 8,
                    },
                    SearchChar {
                        value: wch!($ty, 'g'),
                        count: 8,
                    },
                    SearchChar {
                        value: wch!($ty, 'p'),
                        count: 8,
                    },
                ],
                common: &[
                    SearchChar {
                        value: wch!($ty, 'a'),
                        count: 44,
                    },
                    SearchChar {
                        value: wch!($ty, 'h'),
                        count: 34,
                    },
                    SearchChar {
                        value: wch!($ty, 'i'),
                        count: 35,
                    },
                ],
                very_common: &[SearchChar {
                    value: wch!($ty, ' '),
                    count: 106,
                }],
                super_common: &[],
            };
        )*
    };
}

macro_rules! tiny {
    ($(
        $vis:vis const $name:ident: Input<$ty:ident>;
    )*) => {
        $(
            $vis const $name: Input<$ty> = Input {
                corpus: include_wch!($ty, "../data/sherlock/tiny.txt"),
                never: &[
                    SearchChar {
                        value: wch!($ty, '<'),
                        count: 0,
                    },
                    SearchChar {
                        value: wch!($ty, '>'),
                        count: 0,
                    },
                    SearchChar {
                        value: wch!($ty, '='),
                        count: 0,
                    },
                ],
                rare: &[
                    SearchChar {
                        value: wch!($ty, '.'),
                        count: 1,
                    },
                    SearchChar {
                        value: wch!($ty, 'H'),
                        count: 1,
                    },
                    SearchChar {
                        value: wch!($ty, 'M'),
                        count: 1,
                    },
                ],
                uncommon: &[
                    SearchChar {
                        value: wch!($ty, 'l'),
                        count: 5,
                    },
                    SearchChar {
                        value: wch!($ty, 's'),
                        count: 5,
                    },
                    SearchChar {
                        value: wch!($ty, 'e'),
                        count: 6,
                    },
                ],
                common: &[
                    SearchChar {
                        value: wch!($ty, ' '),
                        count: 11,
                    },
                ],
                very_common: &[],
                super_common: &[],
            };
        )*
    };
}

macro_rules! empty {
    ($(
        $vis:vis const $name:ident: Input<$ty:ident>;
    )*) => {
        $(
            $vis const $name: Input<$ty> = Input {
                corpus: &[],
                never: &[
                    SearchChar {
                        value: wch!($ty, 'a'),
                        count: 0,
                    },
                    SearchChar {
                        value: wch!($ty, 'b'),
                        count: 0,
                    },
                    SearchChar {
                        value: wch!($ty, 'c'),
                        count: 0,
                    },
                ],
                rare: &[],
                uncommon: &[],
                common: &[],
                very_common: &[],
                super_common: &[],
            };
        )*
    };
}

huge! {
    pub const HUGE_U16: Input<u16>;
    pub const HUGE_U32: Input<u32>;
}

small! {
    pub const SMALL_U16: Input<u16>;
    pub const SMALL_U32: Input<u32>;
}

tiny! {
    pub const TINY_U16: Input<u16>;
    pub const TINY_U32: Input<u32>;
}

empty! {
    pub const EMPTY_U16: Input<u16>;
    pub const EMPTY_U32: Input<u32>;
}
