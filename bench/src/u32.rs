use criterion::{Bencher, Criterion};

use crate::count;
use crate::input::{Input, Search};

use crate::input::EMPTY_U32 as EMPTY;
use crate::input::HUGE_U32 as HUGE;
use crate::input::SMALL_U32 as SMALL;
use crate::input::TINY_U32 as TINY;

pub fn all(c: &mut Criterion) {
    macro_rules! def {
        ($group:literal, $fn:path) => {
            define_wmemchr_input(c, concat!($group, "/huge"), HUGE, move |search, b| {
                b.iter(|| assert_eq!(search.value.count, $fn(search.value.value, search.corpus)));
            });
            define_wmemchr_input(c, concat!($group, "/small"), SMALL, move |search, b| {
                b.iter(|| assert_eq!(search.value.count, $fn(search.value.value, search.corpus)));
            });
            define_wmemchr_input(c, concat!($group, "/tiny"), TINY, move |search, b| {
                b.iter(|| assert_eq!(search.value.count, $fn(search.value.value, search.corpus)));
            });
            define_wmemchr_input(c, concat!($group, "/empty"), EMPTY, move |search, b| {
                b.iter(|| assert_eq!(search.value.count, $fn(search.value.value, search.corpus)));
            });
        };
    }
    def!("fallback", count::fallback);
    def!("naive", count::naive);
}

fn define_wmemchr_input(
    c: &mut Criterion,
    group: &str,
    input: Input<u32>,
    bench: impl FnMut(Search<u32>, &mut Bencher<'_>) + Clone + 'static,
) {
    macro_rules! def {
        ($name:literal, $kind:ident) => {
            if let Some(search) = input.$kind() {
                let name = format!("u32/{}/{}", group, $name);
                let mut bench = bench.clone();
                $crate::define(
                    c,
                    &name,
                    input.corpus.len(),
                    Box::new(move |b| bench(search, b)),
                );
            }
        };
    }
    def!("never", never);
    def!("rare", rare);
    def!("uncommon", uncommon);
    def!("common", common);
    def!("very_common", very_common);
    def!("super_common", super_common);
}
