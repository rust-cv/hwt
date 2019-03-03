use criterion::*;
use hwt::*;

fn bench_neighbors(c: &mut Criterion) {
    c.bench(
        "neighbors",
        ParameterizedBenchmark::new(
            "search_radius_2_take_100",
            |bencher: &mut Bencher, &total: &usize| {
                let range = (0..).take(total);
                let mut hwt = Hwt::new();
                for i in range.clone() {
                    hwt.insert(u128::from(i), i, u128::from);
                }
                let mut cycle_range = range.cycle();
                bencher.iter(|| {
                    let feature = cycle_range.next().unwrap();
                    assert_eq!(
                        hwt.neighbors(2, u128::from(feature), &u128::from)
                            .take(100)
                            .count(),
                        100
                    );
                });
            },
            (14..30).map(|n| 2usize.pow(n)),
        )
        .sample_size(30),
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
