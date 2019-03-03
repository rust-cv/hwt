use criterion::*;
use hwt::*;

fn bench_neighbors(c: &mut Criterion) {
    c.bench(
        "neighbors",
        ParameterizedBenchmark::new(
            "search_radius_2",
            |bencher: &mut Bencher, &total: &usize| {
                let range = (0..).take(total);
                let mut hwt = Hwt::new();
                for i in range.clone() {
                    hwt.insert(u128::from(i), i, u128::from);
                }
                let mut cycle_range = range.cycle();
                bencher.iter(|| {
                    let feature = cycle_range.next().unwrap();
                    assert!(hwt.neighbors(2, u128::from(feature), &u128::from).count() < 8128);
                });
            },
            (1..6).map(|n| 10usize.pow(n)),
        ),
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
