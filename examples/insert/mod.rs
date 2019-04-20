use criterion::*;
use hwt::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

fn bench_insert(c: &mut Criterion) {
    let mut rng = SmallRng::from_seed([5; 16]);
    let space = rng
        .sample_iter(&rand::distributions::Standard)
        .take(1 << 20)
        .collect::<Vec<u128>>();
    
    c.bench(
        "insert",
        Benchmark::new("insert_2^20_times", move |bencher: &mut Bencher| {
            let mut hwt = Hwt::new();
            bencher.iter(|| {
                for &f in &space {
                    hwt.insert(f);
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
