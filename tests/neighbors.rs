use hwt::*;

#[test]
fn test_neighbors() {
    let features = [0b1001, 0b1010, 0b1100, 0b1000];
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
}