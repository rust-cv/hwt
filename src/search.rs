use itertools::Itertools;

/// Iterator over the indices that fall within a radius of a number.
///
/// - `bits` - The number of bits the that make up the bit substring `sl`
///     comes from.
/// - `sl` - The weight of the left half of the 2-bit search number.
/// - `sw` - The weight of the whole 2-bit search number.
/// - `tw` - The weight of the whole 2-bit target number.
/// - `radius` - The maximum possible distance of matches.
///
/// Returns the iterator over the indices and also the bucket size
/// (`MIN - MAX`). The iterator always iterates over increasing `SOD`.
pub fn search(
    bits: u32,
    sl: u32,
    sw: u32,
    tw: u32,
    radius: u32,
) -> (impl Iterator<Item = (u32, u32)>, u32) {
    // This function uses things derived in the Search section in the crate
    // documentation. Read that before messing with this code.

    // Compute the `max` and `min` for `tl` range.
    let max = std::cmp::min(tw, bits);
    let min = tw - max;

    // We do a lot of signed operations and sometimes compute negative numbers.
    // It is easier to change these to `i32` now.
    let sl = sl as i32;
    let sw = sw as i32;
    let tw = tw as i32;
    let radius = radius as i32;

    // See crate documentation on what `C` is.
    let c = 2 * sl - sw + tw;

    let filter = move |&tl: &i32| tl >= min as i32 && tl <= max as i32;

    let map = move |tl: i32| {
        (
            tl as u32,
            ((tl - sl).abs() + ((tw - tl) - (sw - sl)).abs()) as u32,
        )
    };

    // Check if we intersect.
    if ((radius + c) / 2 - sl).abs() + (tw - (radius + c) / 2 - sw + sl).abs() <= radius {
        // We do, so run the ranges.
        let start = (-radius + c + 1) / 2;
        let inflection1 = sl;
        let inflection2 = sl - sw + tw;
        let end = (radius + c) / 2;
        let down = start..inflection1;
        let flat = inflection1..=inflection2;
        let up = inflection2 + 1..=end;

        // We interleave `down` and `up` so that the resulting iterator always
        // goes in increasing `SOD` order. `flat` is always the best matches.
        (
            flat.chain(down.interleave(up)).filter(filter).map(map),
            max - min + 1,
        )
    } else {
        // Create fake iterators to satisfy the type system.
        let down = 0..0;
        let flat = 0..=-1;
        let up = 0..=-1;

        // Also perform the same operations over here.
        (
            flat.chain(down.interleave(up)).filter(filter).map(map),
            max - min + 1,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn search_sort(bits: u32, sl: u32, sw: u32, tw: u32, radius: u32) -> (Vec<(u32, u32)>, u32) {
        let indices_sorter = |a: &(u32, u32), b: &(u32, u32)| a.0.cmp(&b.0);
        let (indices, size) = search(bits, sl, sw, tw, radius);
        let mut indices = indices.collect::<Vec<_>>();
        indices.sort_unstable_by(indices_sorter);
        (indices, size)
    }

    #[test]
    fn test_search() {
        let (indices, size) = search_sort(64, 3, 5, 4, 1);
        assert_eq!(&indices, &[(2, 1), (3, 1)]);
        assert_eq!(size, 5);

        let (indices, size) = search_sort(64, 58, 66, 40, 1);
        assert_eq!(&indices, &[]);
        assert_eq!(size, 41);

        // [58, 8] ([sl, sr])
        let (indices, size) = search_sort(64, 58, 66, 66, 1);
        assert_eq!(&indices, &[(58, 0)]);
        assert_eq!(size, 63);

        // [58, 8] ([sl, sr])
        let (indices, size) = search_sort(64, 58, 66, 66, 5);
        assert_eq!(&indices, &[(56, 4), (57, 2), (58, 0), (59, 2), (60, 4)]);
        assert_eq!(size, 63);

        // [58, 14] ([sl, sr])
        let (indices, size) = search_sort(64, 58, 72, 68, 10);
        assert_eq!(
            &indices,
            &[
                (51, 10),
                (52, 8),
                (53, 6),
                (54, 4),
                (55, 4),
                (56, 4),
                (57, 4),
                (58, 4),
                (59, 6),
                (60, 8),
                (61, 10)
            ]
        );
        assert_eq!(size, 61);
    }
}
