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
            let mut cycle_range = range.cycle();
            bencher.iter(|| {
                let feature = cycle_range.next().unwrap();
                assert!(hwt.neighbors(2, u128::from(feature), &u128::from).count() < 8128);
            });
        }),
    );
}

fn config() -> Criterion {
    Criterion::default()
}

criterion_group! {
    name = benches;
    config = config();
    targets = bench_neighbors
}
