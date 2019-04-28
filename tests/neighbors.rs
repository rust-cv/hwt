use chrono::Utc;
use hwt::*;
use log::LevelFilter;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::path::PathBuf;

#[test]
fn test_neighbors() {
    let mut node_queue = NodeQueue::new();
    let mut feature_heap = FeatureHeap::new();
    let features = [
        0b1001,
        0b1010,
        0b1100,
        0b1000,
        0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA,
    ];
    let mut hwt = Hwt::new();
    for &feature in &features {
        hwt.insert(feature);
    }

    for &feature in &features {
        let mut neighbors = [0; 1];
        let neighbors = hwt.nearest(
            feature,
            128,
            0,
            &mut node_queue,
            &mut feature_heap,
            &mut neighbors,
        );
        assert_eq!(neighbors[0], feature);
    }

    let mut neighbors = hwt.search_radius(1, 0b1000).collect::<Vec<u128>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[0b1000, 0b1001, 0b1010, 0b1100]);

    let mut neighbors = hwt.search_radius(1, 0b1001).collect::<Vec<u128>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[0b1000, 0b1001]);

    let mut neighbors = hwt.search_radius(1, 0b1010).collect::<Vec<u128>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[0b1000, 0b1010]);

    let mut neighbors = hwt.search_radius(1, 0b1100).collect::<Vec<u128>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[0b1000, 0b1100]);

    let range = (0..).take(1 << 4);
    let mut hwt = Hwt::new();
    for i in range.clone() {
        hwt.insert(i);
    }
    for feature in range.clone() {
        assert!(hwt.search_radius(2, feature).count() < 8128);
    }
}

#[test]
fn compare_to_linear() -> std::io::Result<()> {
    // Start logging.
    let now = Utc::now();
    let log_dir = PathBuf::from("target").join("logs");
    std::fs::create_dir_all(&log_dir)?;
    let log_file = log_dir.join(now.format("%Z_%F_%H-%M-%S.txt").to_string());
    eprintln!("logging in {}", log_file.display());
    simple_logging::log_to_file(&log_file, LevelFilter::Trace)?;

    let mut node_queue = NodeQueue::new();
    let mut feature_heap = FeatureHeap::new();

    let mut rng = SmallRng::from_seed([5; 16]);
    let space = rng
        .sample_iter(&rand::distributions::Standard)
        .take(80_000)
        .collect::<Vec<u128>>();
    let search = rng
        .sample_iter(&rand::distributions::Standard)
        .take(10)
        .collect::<Vec<u128>>();

    let mut hwt = Hwt::new();
    for &f in &space {
        hwt.insert(f);
    }

    for f0 in search {
        let mut neighbors_err0 = [0; 1];
        let neighbors_err0 = hwt.nearest(
            f0,
            128,
            0,
            &mut node_queue,
            &mut feature_heap,
            &mut neighbors_err0,
        );
        let mut neighbors_err1 = [0; 1];
        let neighbors_err1 = hwt.nearest(
            f0,
            128,
            1,
            &mut node_queue,
            &mut feature_heap,
            &mut neighbors_err1,
        );
        let mut neighbors_err2 = [0; 1];
        let neighbors_err2 = hwt.nearest(
            f0,
            128,
            2,
            &mut node_queue,
            &mut feature_heap,
            &mut neighbors_err2,
        );
        let mut neighbors_err3 = [0; 1];
        let neighbors_err3 = hwt.nearest(
            f0,
            128,
            3,
            &mut node_queue,
            &mut feature_heap,
            &mut neighbors_err3,
        );
        assert_eq!(
            space
                .iter()
                .map(|&f1| (f0 ^ f1).count_ones())
                .min()
                .unwrap(),
            (neighbors_err0[0] ^ f0).count_ones()
        );

        assert!(
            space
                .iter()
                .map(|&f1| (f0 ^ f1).count_ones())
                .min()
                .unwrap()
                + 1
                >= (neighbors_err1[0] ^ f0).count_ones()
        );

        assert!(
            space
                .iter()
                .map(|&f1| (f0 ^ f1).count_ones())
                .min()
                .unwrap()
                + 2
                >= (neighbors_err2[0] ^ f0).count_ones()
        );

        assert!(
            space
                .iter()
                .map(|&f1| (f0 ^ f1).count_ones())
                .min()
                .unwrap()
                + 3
                >= (neighbors_err3[0] ^ f0).count_ones()
        );
    }

    Ok(())
}
