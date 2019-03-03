use criterion::*;
use hwt::*;

fn bench_neighbors(c: &mut Criterion) {
    c.bench(
        "neighbors",
        Benchmark::new("search_2^12_neighbors_radius_2", |bencher: &mut Bencher| {
            let range = (0..).take(1 << 12);
            let mut hwt = Hwt::new();
            for i in range.clone() {
                hwt.insert(u128::from(i), i, u128::from);
            }
            bencher.iter(|| {
                for feature in range.clone() {
                    assert!(hwt.neighbors(2, u128::from(feature), &u128::from).count() < 8128);
                }
            });
        }),
    );
}

fn config() -> Criterion {
    Criterion::default().sample_size(32).nresamples(5)
}

criterion_group! {
    name = benches;
    config = config();
    targets = bench_neighbors
}
