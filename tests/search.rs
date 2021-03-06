use hwt::{indices::*, search::*};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use swar::*;

#[test]
fn test_search2() {
    // Generate some random features.
    let mut rng = SmallRng::from_seed([5; 16]);
    let space = rng
        .sample_iter(&rand::distributions::Standard)
        .take(8_000)
        .collect::<Vec<u128>>();
    let search = rng
        .sample_iter(&rand::distributions::Standard)
        .take(10)
        .collect::<Vec<u128>>();

    for &s in &search {
        let sindices = indices128(s);
        let sp = Bits128(sindices[0]);
        let sc = Bits64(sindices[1]);

        for &t in &space {
            let tindices = indices128(t);
            let tp = Bits128(tindices[0]);
            let tc = Bits64(tindices[1]);

            let distance = (t ^ s).count_ones();
            let child_distance = (sc.0 ^ tc.0).count_ones();

            assert!(
                search_exact2(64, sp, sc, tp, child_distance).any(|exact_tc| exact_tc == tc),
                "sp({:032X}) sc({:032X}) tp({:032X}) distance({}) expected tc({:032X})",
                sp.0,
                sc.0,
                tp.0,
                child_distance,
                tc.0
            );
            for Bits64(exact_tc) in search_exact2(64, sp, sc, tp, child_distance) {
                assert_eq!(
                    (exact_tc ^ sc.0).count_ones(),
                    child_distance,
                    "got {:032X} against {:032X}",
                    exact_tc,
                    sc.0
                );
            }

            assert!(search_radius2(64, sp, sc, tp, distance).any(|(exact_tc, _)| exact_tc == tc));
            assert!(search_radius2(64, sp, sc, tp, distance + 1).any(|(exact_tc, _)| exact_tc == tc));
            for (Bits64(exact_tc), sod) in search_radius2(64, sp, sc, tp, distance) {
                assert_eq!(
                    (exact_tc ^ sc.0).count_ones(),
                    sod,
                    "got {:032X} against {:032X}",
                    exact_tc,
                    sc.0
                );
                assert!(sod <= distance);
            }
        }
    }
}
