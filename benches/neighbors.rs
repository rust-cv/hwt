use criterion::*;
use hwt::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::iter::FromIterator;

fn bench_neighbors(c: &mut Criterion) {
    let max_tree_magnitude = 26;
    let all_sizes = (0..=max_tree_magnitude).map(|n| 2usize.pow(n));
    let mut rng = SmallRng::from_seed([5; 16]);
    // Get the bigest input size and then generate all inputs from that.
    eprintln!("Generating random inputs...");
    let all_input = rng
        .sample_iter(&rand::distributions::Standard)
        .take(all_sizes.clone().rev().next().unwrap())
        .collect::<Vec<u128>>();
    eprintln!("Done.");
    eprintln!("Generating Hamming Weight Trees...");
    let hwt_map = HashMap::<_, _>::from_iter(all_sizes.clone().map(|total| {
        eprintln!("Generating tree size {}...", total);
        let range = (0..).take(total);
        let mut hwt = Hwt::new();
        for i in range.clone() {
            hwt.insert(all_input[i], i as u32, |n| all_input[n as usize]);
        }
        (total, hwt)
    }));
    eprintln!("Done.");
    c.bench(
        "neighbors",
        ParameterizedBenchmark::new(
            "nearest_neighbor",
            move |bencher: &mut Bencher, total: &usize| {
                let hwt = &hwt_map[total];
                let mut cycle_range = (0..).take(*total).cycle();
                bencher.iter(|| {
                    let feature = cycle_range.next().unwrap();
                    assert_eq!(
                        hwt.nearest(all_input[feature], &|n| all_input[n as usize])
                            .take(1)
                            .count(),
                        1
                    );
                });
            },
            all_sizes,
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
