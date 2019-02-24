use criterion::*;
use hwt::*;

fn bench_insert(c: &mut Criterion) {
    c.bench(
        "insert",
        Benchmark::new("insert_2^20_times", |bencher: &mut Bencher| {
            let mut range = 0..;
            let mut hwt = Hwt::new();
            bencher.iter(|| {
                for i in (&mut range).take(1 << 20) {
                    hwt.insert(u128::from(i), i, u128::from);
                }
            });
        })
        .throughput(Throughput::Elements(1 << 20)),
    );
}

fn config() -> Criterion {
    Criterion::default().sample_size(32).nresamples(5)
}

criterion_group! {
    name = benches;
    config = config();
    targets = bench_insert
}
