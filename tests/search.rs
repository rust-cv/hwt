use hwt::{indices::*, search::*};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use swar::*;

#[test]
fn test2() {
    // Generate some random features.
    let mut rng = SmallRng::from_seed([5; 16]);
    let space = rng
        .sample_iter(&rand::distributions::Standard)
        .take(80_000)
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

            assert!(search_exact2(sp, sc, tp, distance).any(|exact_tc| exact_tc == tc));
            for Bits64(exact_tc) in search_exact2(sp, sc, tp, distance) {
                assert_eq!(
                    (exact_tc ^ sc.0).count_ones(),
                    distance,
                    "got {:032X} against {:032X}",
                    exact_tc,
                    sc.0
                );
            }
        }
    }
}
