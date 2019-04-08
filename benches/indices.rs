use criterion::*;
use hwt::indices::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

fn bench_indices(c: &mut Criterion) {
    let mut rng = SmallRng::from_seed([5; 16]);
    let original_input = rng
        .sample_iter(&rand::distributions::Standard)
        .take(1 << 10)
        .collect::<Vec<u128>>();

    let input = original_input.clone();
    c.bench(
        "indices_2^10_samples",
        Benchmark::new("indices2", move |bencher: &mut Bencher| {
            bencher.iter(|| {
                input
                    .iter()
                    .cloned()
                    .map(|i| indices2(i as u8 & 0b11).0[0])
                    .sum::<usize>()
            });
        })
        .throughput(Throughput::Elements(1 << 10)),
    );

    let input = original_input.clone();
    c.bench(
        "indices_2^10_samples",
        Benchmark::new("indices4", move |bencher: &mut Bencher| {
            bencher.iter(|| {
                input
                    .iter()
                    .cloned()
                    .map(|i| indices4(i as u8 & 0b1111).0[1])
                    .sum::<usize>()
            });
        })
        .throughput(Throughput::Elements(1 << 10)),
    );

    let input = original_input.clone();
    c.bench(
        "indices_2^10_samples",
        Benchmark::new("indices8", move |bencher: &mut Bencher| {
            bencher.iter(|| {
                input
                    .iter()
                    .cloned()
                    .map(|i| indices8(i as u8).0[2])
                    .sum::<usize>()
            });
        })
        .throughput(Throughput::Elements(1 << 10)),
    );

    let input = original_input.clone();
    c.bench(
        "indices_2^10_samples",
        Benchmark::new("indices16", move |bencher: &mut Bencher| {
            bencher.iter(|| {
                input
                    .iter()
                    .cloned()
                    .map(|i| indices16(i as u16).0[3])
                    .sum::<usize>()
            });
        })
        .throughput(Throughput::Elements(1 << 10)),
    );

    let input = original_input.clone();
    c.bench(
        "indices_2^10_samples",
        Benchmark::new("indices32", move |bencher: &mut Bencher| {
            bencher.iter(|| {
                input
                    .iter()
                    .cloned()
                    .map(|i| indices32(i as u32).0[4])
                    .sum::<usize>()
            });
        })
        .throughput(Throughput::Elements(1 << 10)),
    );

    let input = original_input.clone();
    c.bench(
        "indices_2^10_samples",
        Benchmark::new("indices64", move |bencher: &mut Bencher| {
            bencher.iter(|| {
                input
                    .iter()
                    .cloned()
                    .map(|i| indices64(i as u64).0[5])
                    .sum::<usize>()
            });
        })
        .throughput(Throughput::Elements(1 << 10)),
    );

    let input = original_input.clone();
    c.bench(
        "indices_2^10_samples",
        Benchmark::new("indices128", move |bencher: &mut Bencher| {
            bencher.iter(|| {
                input
                    .iter()
                    .cloned()
                    .map(|i| indices128(i as u128)[6])
                    .sum::<usize>()
            });
        })
        .throughput(Throughput::Elements(1 << 10)),
    );
}

fn config() -> Criterion {
    Criterion::default().sample_size(30)
}

criterion_group! {
    name = benches;
    config = config();
    targets = bench_indices
}
