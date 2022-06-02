use criterion::{black_box, criterion_group, criterion_main, Criterion};
use xmp::UpdateResults;
pub fn write_xmp(up: UpdateResults) {
    up.write_to("tests/__file.xmp").unwrap();
}

pub fn write_jpeg(up: UpdateResults) {
    up.write_to("tests/__file.jpg").unwrap();
}

fn write_benchmark(c: &mut Criterion) {
    std::fs::remove_dir("tests").ok();
    std::fs::create_dir("tests").ok();

    std::fs::copy("assets/file.xmp", "tests/__file.xmp").ok();
    std::fs::copy("assets/1.jpg", "tests/__file.jpg").ok();

    c.bench_function("write_xmp", |b| {
        b.iter(|| write_xmp(black_box(UpdateResults::default())))
    });
    c.bench_function("write_jpg", |b| {
        b.iter(|| write_jpeg(black_box(UpdateResults::default())))
    });
}

criterion_group!(benches, write_benchmark);
criterion_main!(benches);
