use criterion::*;
use hwt::indices::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

fn bench_indices(c: &mut Criterion) {
    let mut rng = SmallRng::from_seed([5; 16]);
    c.bench_functions(
        "indices_over_32768_samples",
        vec![
            Fun::new("indices2", |bencher: &mut Bencher, input: &Vec<u128>| {
                bencher.iter(|| {
                    input
                        .iter()
                        .cloned()
                        .map(|i| indices2(i as u8 & 0b11).0[0])
                        .sum::<usize>()
                });
            }),
            Fun::new("indices4", |bencher: &mut Bencher, input: &Vec<u128>| {
                bencher.iter(|| {
                    input
                        .iter()
                        .cloned()
                        .map(|i| indices4(i as u8 & 0b1111).0[0])
                        .sum::<usize>()
                });
            }),
            Fun::new("indices8", |bencher: &mut Bencher, input: &Vec<u128>| {
                bencher.iter(|| {
                    input
                        .iter()
                        .cloned()
                        .map(|i| indices8(i as u8).0[0])
                        .sum::<usize>()
                });
            }),
            Fun::new("indices16", |bencher: &mut Bencher, input: &Vec<u128>| {
                bencher.iter(|| {
                    input
                        .iter()
                        .cloned()
                        .map(|i| indices16(i as u16).0[0])
                        .sum::<usize>()
                });
            }),
            Fun::new("indices32", |bencher: &mut Bencher, input: &Vec<u128>| {
                bencher.iter(|| {
                    input
                        .iter()
                        .cloned()
                        .map(|i| indices32(i as u32).0[0])
                        .sum::<usize>()
                });
            }),
            Fun::new("indices64", |bencher: &mut Bencher, input: &Vec<u128>| {
                bencher.iter(|| {
                    input
                        .iter()
                        .cloned()
                        .map(|i| indices64(i as u64).0[0])
                        .sum::<usize>()
                });
            }),
            Fun::new("indices128", |bencher: &mut Bencher, input: &Vec<u128>| {
                bencher.iter(|| {
                    input
                        .iter()
                        .cloned()
                        .map(|i| indices128(i as u128).0[0])
                        .sum::<usize>()
                });
            }),
        ],
        rng.sample_iter(&rand::distributions::Standard)
            .take(32768)
            .collect::<Vec<u128>>(),
    );
}

fn config() -> Criterion {
    Criterion::default()
}

criterion_group! {
    name = benches;
    config = config();
    targets = bench_indices
}
