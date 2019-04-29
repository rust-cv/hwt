use criterion::*;
use hwt::*;
use rand::distributions::{Bernoulli, Standard};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::rc::Rc;

/// This is the probability each bit of an inlier will be different.
/// This comes from "Online Nearest Neighbor Search in Hamming Space"
/// in figure 2a, where 1-NN has an average search radius of 11 for
/// 128-bit features. I make the assumption here that the inliers
/// exist on a binomial distribution over 128 choices centered at
/// 11, which is consistent with the inlier statistics found in the paper
/// "ORB: an efficient alternative to SIFT or SURF".
const BIT_DIFF_PROBABILITY_OF_INLIER: f64 = 0.0859;

fn bench_neighbors(c: &mut Criterion) {
    let space_mags = 0..=24;
    let all_sizes = (space_mags).map(|n| 2usize.pow(n));
    let mut rng = SmallRng::from_seed([5; 16]);
    // Get the bigest input size and then generate all inputs from that.
    eprintln!("Generating random inputs...");
    let all_input = rng
        .sample_iter(&Standard)
        .take(all_sizes.clone().rev().next().unwrap())
        .collect::<Vec<u128>>();
    let linear_all_input = all_input.clone();
    eprintln!("Done.");
    eprintln!("Generating Hamming Weight Trees...");
    let bernoulli = Bernoulli::new(BIT_DIFF_PROBABILITY_OF_INLIER);
    let hwt_map = Rc::new(HashMap::<_, _>::from_iter(all_sizes.clone().map(|total| {
        eprintln!("Generating tree size {}...", total);
        let range = 0..total;
        let mut hwt = Hwt::new();
        for i in range.clone() {
            hwt.insert(all_input[i]);
        }
        // In the paper they choose 1000 samples that arent in the data set.
        let inliers: Vec<u128> = all_input[0..total]
            .choose_multiple(&mut rng, 1000)
            .map(|&feature| {
                let mut feature = feature;
                for bit in 0..128 {
                    let choice: bool = rng.sample(&bernoulli);
                    feature ^= (choice as u128) << bit;
                }
                feature
            })
            .collect();
        (total, (hwt, inliers))
    })));
    let linear_hwt_map = hwt_map.clone();
    let maxerr_1_hwt_map = hwt_map.clone();
    let maxerr_2_hwt_map = hwt_map.clone();
    let maxerr_3_hwt_map = hwt_map.clone();
    let maxerr_5_hwt_map = hwt_map.clone();
    let maxerr_10_hwt_map = hwt_map.clone();
    eprintln!("Done.");
    c.bench(
        "neighbors",
        ParameterizedBenchmark::new(
            "nearest_1_hwt_maxerr_0",
            move |bencher: &mut Bencher, total: &usize| {
                let (hwt, inliers) = &hwt_map[total];
                let mut cycle_range = inliers.iter().cloned().cycle();
                let mut node_queue = NodeQueue::new();
                let mut feature_heap = FeatureHeap::new();
                bencher.iter(|| {
                    let feature = cycle_range.next().unwrap();
                    let mut neighbors = [0; 1];
                    hwt.nearest(
                        feature,
                        128,
                        0,
                        &mut node_queue,
                        &mut feature_heap,
                        &mut neighbors,
                    )
                    .len()
                });
            },
            all_sizes,
        )
        .with_function(
            "nearest_1_hwt_maxerr_1",
            move |bencher: &mut Bencher, total: &usize| {
                let (hwt, inliers) = &maxerr_1_hwt_map[total];
                let mut cycle_range = inliers.iter().cloned().cycle();
                let mut node_queue = NodeQueue::new();
                let mut feature_heap = FeatureHeap::new();
                bencher.iter(|| {
                    let feature = cycle_range.next().unwrap();
                    let mut neighbors = [0; 1];
                    hwt.nearest(
                        feature,
                        128,
                        1,
                        &mut node_queue,
                        &mut feature_heap,
                        &mut neighbors,
                    )
                    .len()
                });
            },
        )
        .with_function(
            "nearest_1_hwt_maxerr_2",
            move |bencher: &mut Bencher, total: &usize| {
                let (hwt, inliers) = &maxerr_2_hwt_map[total];
                let mut cycle_range = inliers.iter().cloned().cycle();
                let mut node_queue = NodeQueue::new();
                let mut feature_heap = FeatureHeap::new();
                bencher.iter(|| {
                    let feature = cycle_range.next().unwrap();
                    let mut neighbors = [0; 1];
                    hwt.nearest(
                        feature,
                        128,
                        2,
                        &mut node_queue,
                        &mut feature_heap,
                        &mut neighbors,
                    )
                    .len()
                });
            },
        )
        .with_function(
            "nearest_1_hwt_maxerr_3",
            move |bencher: &mut Bencher, total: &usize| {
                let (hwt, inliers) = &maxerr_3_hwt_map[total];
                let mut cycle_range = inliers.iter().cloned().cycle();
                let mut node_queue = NodeQueue::new();
                let mut feature_heap = FeatureHeap::new();
                bencher.iter(|| {
                    let feature = cycle_range.next().unwrap();
                    let mut neighbors = [0; 1];
                    hwt.nearest(
                        feature,
                        128,
                        3,
                        &mut node_queue,
                        &mut feature_heap,
                        &mut neighbors,
                    )
                    .len()
                });
            },
        )
        .with_function(
            "nearest_1_hwt_maxerr_5",
            move |bencher: &mut Bencher, total: &usize| {
                let (hwt, inliers) = &maxerr_5_hwt_map[total];
                let mut cycle_range = inliers.iter().cloned().cycle();
                let mut node_queue = NodeQueue::new();
                let mut feature_heap = FeatureHeap::new();
                bencher.iter(|| {
                    let feature = cycle_range.next().unwrap();
                    let mut neighbors = [0; 1];
                    hwt.nearest(
                        feature,
                        128,
                        5,
                        &mut node_queue,
                        &mut feature_heap,
                        &mut neighbors,
                    )
                    .len()
                });
            },
        )
        .with_function(
            "nearest_1_hwt_maxerr_10",
            move |bencher: &mut Bencher, total: &usize| {
                let (hwt, inliers) = &maxerr_10_hwt_map[total];
                let mut cycle_range = inliers.iter().cloned().cycle();
                let mut node_queue = NodeQueue::new();
                let mut feature_heap = FeatureHeap::new();
                bencher.iter(|| {
                    let feature = cycle_range.next().unwrap();
                    let mut neighbors = [0; 1];
                    hwt.nearest(
                        feature,
                        128,
                        10,
                        &mut node_queue,
                        &mut feature_heap,
                        &mut neighbors,
                    )
                    .len()
                });
            },
        )
        .with_function(
            "nearest_1_linear",
            move |bencher: &mut Bencher, &total: &usize| {
                let (_, inliers) = &linear_hwt_map[&total];
                let mut cycle_range = inliers.iter().cloned().cycle();
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
