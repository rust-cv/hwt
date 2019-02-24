use criterion::*;
use hwt::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

fn bench_insert(c: &mut Criterion) {
    let mut rng = SmallRng::from_seed([5; 16]);
    let input = rng
        .sample_iter(&rand::distributions::Standard)
        .take(1 << 10)
        .collect::<Vec<u128>>();
    c.bench_function("insert_1024_times", move |bencher: &mut Bencher| {
        bencher.iter(|| {
            let mut hwt = Hwt::new();
            for (ix, &i) in input.iter().enumerate() {
                hwt.insert(i, ix as u32, |n| input[n as usize]);
            }
        });
    });
}

fn config() -> Criterion {
    Criterion::default()
}

criterion_group! {
    name = benches;
    config = config();
    targets = bench_insert
}
