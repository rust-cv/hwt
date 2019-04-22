use criterion::*;
use hwt::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::iter::FromIterator;

fn bench_neighbors(c: &mut Criterion) {
    let space_mags = 24..=24;
    let all_sizes = (space_mags).map(|n| 2usize.pow(n));
    let mut rng = SmallRng::from_seed([5; 16]);
    // Get the bigest input size and then generate all inputs from that.
    eprintln!("Generating random inputs...");
    let all_input = rng
        .sample_iter(&rand::distributions::Standard)
        .take(all_sizes.clone().rev().next().unwrap())
        .collect::<Vec<u128>>();
    let linear_all_input = all_input.clone();
    // Sample 10000 random features for lookups.
    let random_samples = rng
        .sample_iter(&rand::distributions::Standard)
        .take(10000)
        .collect::<Vec<u128>>();
    let linear_random_samples = random_samples.clone();
    eprintln!("Done.");
    eprintln!("Generating Hamming Weight Trees...");
    let hwt_map = HashMap::<_, _>::from_iter(all_sizes.clone().map(|total| {
        eprintln!("Generating tree size {}...", total);
        let range = (0..).take(total);
        let mut hwt = Hwt::new();
        for i in range.clone() {
            hwt.insert(all_input[i]);
        }
        (total, hwt)
    }));
    eprintln!("Done.");
    c.bench(
        "neighbors",
        ParameterizedBenchmark::new(
            "nearest_1_hwt",
            move |bencher: &mut Bencher, total: &usize| {
                let hwt = &hwt_map[total];
                let mut cycle_range = random_samples.iter().cloned().cycle();
                bencher.iter(|| {
                    let feature = cycle_range.next().unwrap();
                    let mut neighbors = [0; 1];
                    assert_eq!(hwt.nearest(feature, &mut neighbors).len(), 1);
                });
            },
            all_sizes,
        )
        .with_function(
            "nearest_1_linear",
            move |bencher: &mut Bencher, &total: &usize| {
                let mut cycle_range = linear_random_samples.iter().cloned().cycle();
                bencher.iter(|| {
                    let feature = cycle_range.next().unwrap();
                    linear_all_input[0..total]
                        .iter()
                        .cloned()
                        .min_by_key(|n| (feature ^ n).count_ones())
                });
            },
        ),
    );
}

fn config() -> Criterion {
    Criterion::default().sample_size(32)
}

criterion_group! {
    name = benches;
    config = config();
    targets = bench_neighbors
}
