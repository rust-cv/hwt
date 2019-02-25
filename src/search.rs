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

    let tl_filter = move |&tl: &i32| tl >= min as i32 && tl <= max as i32;

    // We do a lot of signed operations and sometimes compute negative numbers.
    // It is easier to change these to `i32` now.
    let sl = sl as i32;
    let sw = sw as i32;
    let tw = tw as i32;
    let radius = radius as i32;

    // See crate documentation on what `C` is.
    let c = 2 * sl - sw + tw;

    let down_map = move |tl| (tl, -2 * tl + c);
    let flat_map = move |tl| (tl, -sw + tw);
    let up_map = move |tl| (tl, 2 * tl - c);

    let min_map = move |(tl, sod)| (tl as u32 - min, sod as u32);

    // Check if we intersect.
    if ((radius + c) / 2 - sl).abs() + (tw - (radius + c) / 2 - sw + sl).abs() <= radius {
        // We do, so run the ranges.
        let start = (-radius + c) / 2;
        let inflection1 = sl;
        let inflection2 = sl - sw + tw;
        let end = (radius + c) / 2;
        let down = (start..inflection1).filter(tl_filter).map(down_map);
        let flat = (inflection1..=inflection2).filter(tl_filter).map(flat_map);
        let up = (inflection2 + 1..=end).filter(tl_filter).map(up_map);

        // We interleave `down` and `up` so that the resulting iterator always
        // goes in increasing `SOD` order. `flat` is always the best matches.
        (flat.chain(down.interleave(up)).map(min_map), max - min + 1)
    } else {
        // Create fake iterators to satisfy the type system.
        let down = (0..0).filter(tl_filter).map(down_map);
        let flat = (0..=-1).filter(tl_filter).map(flat_map);
        let up = (0..=-1).filter(tl_filter).map(up_map);

        // Also perform the same operations over here.
        (flat.chain(down.interleave(up)).map(min_map), max - min + 1)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_search() {
        let (indices, size) = search(64, 3, 5, 4, 1);
        assert_eq!(&indices.collect::<Vec<_>>(), &[(2, 1), (3, 1)]);
        assert_eq!(size, 5);

        let (indices, size) = search(64, 58, 66, 40, 1);
        assert_eq!(&indices.collect::<Vec<_>>(), &[]);
        assert_eq!(size, 41);
    }
}
