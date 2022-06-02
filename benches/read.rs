use criterion::{criterion_group, criterion_main, Criterion};
use xmp::OptionalResults;
pub fn read_xmp() {
    OptionalResults::load("assets/file.xmp").unwrap();
}

pub fn read_jpeg() {
    OptionalResults::load("assets/1.jpg").unwrap();
}

fn read_benchmark(c: &mut Criterion) {
    c.bench_function("read_xmp", |b| b.iter(read_xmp));
    c.bench_function("read_jpg", |b| b.iter(read_jpeg));
}

criterion_group!(benches, read_benchmark);
criterion_main!(benches);
