use hwt::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[test]
fn test_neighbors() {
    // That number triggers an overflow because the
    // bucket size is precisely as large as `usize`.
    let features = [
        0b1001,
        0b1010,
        0b1100,
        0b1000,
        0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA,
    ];
    let lookup = |n| features[n as usize];
    let mut hwt = Hwt::new();
    for (ix, &feature) in features.iter().enumerate() {
        hwt.insert(feature, ix as u32, lookup);
    }

    for (ix, &feature) in features.iter().enumerate() {
        let mut neighbors = hwt.nearest(feature, &lookup).take(1).collect::<Vec<u32>>();
        neighbors.sort_unstable();
        assert_eq!(&neighbors, &[ix as u32]);
    }

    let mut neighbors = hwt.search_radius(1, 0b1000, &lookup).collect::<Vec<u32>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[0, 1, 2, 3]);

    let mut neighbors = hwt.search_radius(1, 0b1001, &lookup).collect::<Vec<u32>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[0, 3]);

    let mut neighbors = hwt.search_radius(1, 0b1010, &lookup).collect::<Vec<u32>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[1, 3]);

    let mut neighbors = hwt.search_radius(1, 0b1100, &lookup).collect::<Vec<u32>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[2, 3]);

    let range = (0..).take(1 << 4);
    let mut hwt = Hwt::new();
    for i in range.clone() {
        hwt.insert(u128::from(i), i, u128::from);
    }
    for feature in range.clone() {
        assert!(
            hwt.search_radius(2, u128::from(feature), &u128::from)
                .count()
                < 8128
        );
    }
}

#[test]
fn compare_to_linear() {
    let mut rng = SmallRng::from_seed([5; 16]);
    let space = rng
        .sample_iter(&rand::distributions::Standard)
        .take(100_000)
        .collect::<Vec<u128>>();
    let search = rng
        .sample_iter(&rand::distributions::Standard)
        .take(1000)
        .collect::<Vec<u128>>();
    let lookup = |n: u32| space[n as usize];
    
    let mut hwt = Hwt::new();
    for (ix, &f) in space.iter().enumerate() {
        hwt.insert(f, ix as u32, lookup);
    }

    for f0 in search {
        assert_eq!(
            space.iter().map(|&f1| (f0 ^ f1).count_ones()).min(),
            hwt.nearest(f0, &lookup).next()
        );
    }
}
