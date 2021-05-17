use std::time::Duration;

use criterion::{criterion_group, criterion_main, Bencher, Criterion, Throughput};

mod count;
mod input;
mod u16;
mod u32;

fn all(c: &mut Criterion) {
    u16::all(c);
    u32::all(c);
}

fn define(
    c: &mut Criterion,
    name: &str,
    len: usize,
    bench: Box<dyn FnMut(&mut Bencher<'_>) + 'static>,
) {
    let mut iter = name.splitn(2, "/");
    let group_name = iter.next().unwrap();
    let bench_name = iter.next().unwrap();
    c.benchmark_group(group_name)
        .throughput(Throughput::Elements(len as u64))
        .sample_size(10)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2))
        .bench_function(bench_name, bench);
}

criterion_group!(bench, all);
criterion_main!(bench);
