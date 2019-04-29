use criterion::*;
use hwt::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

fn bench_insert(c: &mut Criterion) {
    let space_mags = 0..=22;
    let all_sizes = (space_mags).map(|n| 2usize.pow(n));
    let mut rng = SmallRng::from_seed([5; 16]);
    // Get the bigest input size and then generate all inputs from that.
    eprintln!("Generating random inputs...");
    let all_input = rng
        .sample_iter(&rand::distributions::Standard)
        .take(all_sizes.clone().rev().next().unwrap())
        .collect::<Vec<u128>>();
    eprintln!("Done.");
    c.bench(
        "insert",
        ParameterizedBenchmark::new(
            "times",
            move |bencher: &mut Bencher, &total: &usize| {
                let input = all_input[0..total].iter().cloned();
                bencher.iter(|| {
                    let mut hwt = Hwt::new();
                    for feature in input.clone() {
                        hwt.insert(feature);
                    }
                });
            },
            all_sizes,
        )
        .throughput(|&n| Throughput::Elements(n as u32)),
    );
}

fn config() -> Criterion {
    Criterion::default().sample_size(32)
}

criterion_group! {
    name = benches;
    config = config();
    targets = bench_insert
}
