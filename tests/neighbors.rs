use hwt::*;

#[test]
fn test_neighbors() {
    // That number triggers an overflow because the
    // bucket size is precisely as large as `usize`.
    let features = [0b1001, 0b1010, 0b1100, 0b1000/*, 0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA*/];
    let lookup = |n| features[n as usize];
    let mut hwt = Hwt::new();
    for (ix, &feature) in features.iter().enumerate() {
        hwt.insert(feature, ix as u32, lookup);
    }

    for (ix, &feature) in features.iter().enumerate() {
        let mut neighbors = hwt.neighbors(0, feature, &lookup).collect::<Vec<u32>>();
        neighbors.sort_unstable();
        assert_eq!(&neighbors, &[ix as u32]);
    }

    let mut neighbors = hwt.neighbors(1, 0b1000, &lookup).collect::<Vec<u32>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[0, 1, 2, 3]);

    let mut neighbors = hwt.neighbors(1, 0b1001, &lookup).collect::<Vec<u32>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[0, 3]);

    let mut neighbors = hwt.neighbors(1, 0b1010, &lookup).collect::<Vec<u32>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[1, 3]);

    let mut neighbors = hwt.neighbors(1, 0b1100, &lookup).collect::<Vec<u32>>();
    neighbors.sort_unstable();
    assert_eq!(&neighbors, &[2, 3]);
}