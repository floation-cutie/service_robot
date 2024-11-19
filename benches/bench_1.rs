use criterion::{criterion_group, criterion_main, Criterion};

fn bench_1(c: &mut Criterion) {
    c.bench_function("bench_1", |b| {
        b.iter(|| {
            let mut v = Vec::new();
            for i in 0..1000 {
                v.push(i);
            }
        })
    });
}

criterion_group!(benches, bench_1);
criterion_main!(benches);
