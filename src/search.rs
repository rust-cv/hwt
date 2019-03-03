use itertools::Itertools;

/// Compute the bucket size for an array of `64` `tws` from `search64`.
pub fn compute_bucket_len(tws: [u32; 64]) -> usize {
    let total_diffs: u32 = tws.iter().map(|&tw| (tw & 1) ^ (tw >> 1)).sum();
    // If its greater than 32 then we probably allocated a way too huge bucket.
    assert!(total_diffs < 32);
    1 << total_diffs
}

/// Searches the `128` substrings of a `feature`.
///
/// Bits is assumed to be `1`.
///
/// The target weights `tws` must be known as well.
pub fn search128(feature: u128, tws: [u32; 64], radius: u32) -> impl Iterator<Item = usize> {
    const NPAIRS: u32 = 64;
    // Get the mask for the substring couples.
    let mask = (1u128 << NPAIRS) - 1;
    // Split the `feature` into an array of substrings.
    let substrings = [feature & mask, feature >> NPAIRS];

    let low_indices = search64(
        1,
        substrings[0],
        [
            tws[0], tws[1], tws[2], tws[3], tws[4], tws[5], tws[6], tws[7], tws[8], tws[9],
            tws[10], tws[11], tws[12], tws[13], tws[14], tws[15], tws[16], tws[17], tws[18],
            tws[19], tws[20], tws[21], tws[22], tws[23], tws[24], tws[25], tws[26], tws[27],
            tws[28], tws[29], tws[30], tws[31],
        ],
        radius,
    );
    low_indices.flat_map(move |(low_index, low_sod, low_bucket_size, _)| {
        let high_indices = search64(
            1,
            substrings[1],
            [
                tws[32], tws[33], tws[34], tws[35], tws[36], tws[37], tws[38], tws[39], tws[40],
                tws[41], tws[42], tws[43], tws[44], tws[45], tws[46], tws[47], tws[48], tws[49],
                tws[50], tws[51], tws[52], tws[53], tws[54], tws[55], tws[56], tws[57], tws[58],
                tws[59], tws[60], tws[61], tws[62], tws[63],
            ],
            radius - low_sod,
        );
        high_indices.map(move |(high_index, _, _, _)| high_index * low_bucket_size + low_index)
    })
}

/// Searches the `64` substrings with width `bits` of a `feature`.
///
/// The target weights `tws` must be known as well.
pub fn search64(
    bits: u32,
    feature: u128,
    tws: [u32; 32],
    radius: u32,
) -> impl Iterator<Item = (usize, u32, usize, [u32; 64])> {
    const NPAIRS: u32 = 32;
    // Get the mask for the substring couples.
    let mask = (1u128 << (bits * NPAIRS)) - 1;
    // Split the `feature` into an array of substrings.
    let substrings = [feature & mask, feature >> (NPAIRS * bits)];

    let low_indices = search32(
        bits,
        substrings[0],
        [
            tws[0], tws[1], tws[2], tws[3], tws[4], tws[5], tws[6], tws[7], tws[8], tws[9],
            tws[10], tws[11], tws[12], tws[13], tws[14], tws[15],
        ],
        radius,
    );
    low_indices.flat_map(move |(low_index, low_sod, low_bucket_size, low_tws)| {
        let high_indices = search32(
            bits,
            substrings[1],
            [
                tws[16], tws[17], tws[18], tws[19], tws[20], tws[21], tws[22], tws[23], tws[24],
                tws[25], tws[26], tws[27], tws[28], tws[29], tws[30], tws[31],
            ],
            radius - low_sod,
        );
        high_indices.map(move |(high_index, high_sod, high_bucket_size, high_tws)| {
            (
                high_index * low_bucket_size + low_index,
                low_sod + high_sod,
                low_bucket_size * high_bucket_size,
                [
                    low_tws[0],
                    low_tws[1],
                    low_tws[2],
                    low_tws[3],
                    low_tws[4],
                    low_tws[5],
                    low_tws[6],
                    low_tws[7],
                    low_tws[8],
                    low_tws[9],
                    low_tws[10],
                    low_tws[11],
                    low_tws[12],
                    low_tws[13],
                    low_tws[14],
                    low_tws[15],
                    low_tws[16],
                    low_tws[17],
                    low_tws[18],
                    low_tws[19],
                    low_tws[20],
                    low_tws[21],
                    low_tws[22],
                    low_tws[23],
                    low_tws[24],
                    low_tws[25],
                    low_tws[26],
                    low_tws[27],
                    low_tws[28],
                    low_tws[29],
                    low_tws[30],
                    low_tws[31],
                    high_tws[0],
                    high_tws[1],
                    high_tws[2],
                    high_tws[3],
                    high_tws[4],
                    high_tws[5],
                    high_tws[6],
                    high_tws[7],
                    high_tws[8],
                    high_tws[9],
                    high_tws[10],
                    high_tws[11],
                    high_tws[12],
                    high_tws[13],
                    high_tws[14],
                    high_tws[15],
                    high_tws[16],
                    high_tws[17],
                    high_tws[18],
                    high_tws[19],
                    high_tws[20],
                    high_tws[21],
                    high_tws[22],
                    high_tws[23],
                    high_tws[24],
                    high_tws[25],
                    high_tws[26],
                    high_tws[27],
                    high_tws[28],
                    high_tws[29],
                    high_tws[30],
                    high_tws[31],
                ],
            )
        })
    })
}

/// Searches the `32` substrings with width `bits` of a `feature`.
///
/// The target weights `tws` must be known as well.
pub fn search32(
    bits: u32,
    feature: u128,
    tws: [u32; 16],
    radius: u32,
) -> impl Iterator<Item = (usize, u32, usize, [u32; 32])> {
    const NPAIRS: u32 = 16;
    // Get the mask for the substring couples.
    let mask = (1u128 << (bits * NPAIRS)) - 1;
    // Split the `feature` into an array of substrings.
    let substrings = [feature & mask, feature >> (NPAIRS * bits)];

    let low_indices = search16(
        bits,
        substrings[0],
        [
            tws[0], tws[1], tws[2], tws[3], tws[4], tws[5], tws[6], tws[7],
        ],
        radius,
    );
    low_indices.flat_map(move |(low_index, low_sod, low_bucket_size, low_tws)| {
        let high_indices = search16(
            bits,
            substrings[1],
            [
                tws[8], tws[9], tws[10], tws[11], tws[12], tws[13], tws[14], tws[15],
            ],
            radius - low_sod,
        );
        high_indices.map(move |(high_index, high_sod, high_bucket_size, high_tws)| {
            (
                high_index * low_bucket_size + low_index,
                low_sod + high_sod,
                low_bucket_size * high_bucket_size,
                [
                    low_tws[0],
                    low_tws[1],
                    low_tws[2],
                    low_tws[3],
                    low_tws[4],
                    low_tws[5],
                    low_tws[6],
                    low_tws[7],
                    low_tws[8],
                    low_tws[9],
                    low_tws[10],
                    low_tws[11],
                    low_tws[12],
                    low_tws[13],
                    low_tws[14],
                    low_tws[15],
                    high_tws[0],
                    high_tws[1],
                    high_tws[2],
                    high_tws[3],
                    high_tws[4],
                    high_tws[5],
                    high_tws[6],
                    high_tws[7],
                    high_tws[8],
                    high_tws[9],
                    high_tws[10],
                    high_tws[11],
                    high_tws[12],
                    high_tws[13],
                    high_tws[14],
                    high_tws[15],
                ],
            )
        })
    })
}

/// Searches the `16` substrings with width `bits` of a `feature`.
///
/// The target weights `tws` must be known as well.
pub fn search16(
    bits: u32,
    feature: u128,
    tws: [u32; 8],
    radius: u32,
) -> impl Iterator<Item = (usize, u32, usize, [u32; 16])> {
    const NPAIRS: u32 = 8;
    // Get the mask for the substring couples.
    let mask = (1u128 << (bits * NPAIRS)) - 1;
    // Split the `feature` into an array of substrings.
    let substrings = [feature & mask, feature >> (NPAIRS * bits)];

    let low_indices = search8(
        bits,
        substrings[0],
        [tws[0], tws[1], tws[2], tws[3]],
        radius,
    );
    low_indices.flat_map(move |(low_index, low_sod, low_bucket_size, low_tws)| {
        let high_indices = search8(
            bits,
            substrings[1],
            [tws[4], tws[5], tws[6], tws[7]],
            radius - low_sod,
        );
        high_indices.map(move |(high_index, high_sod, high_bucket_size, high_tws)| {
            (
                high_index * low_bucket_size + low_index,
                low_sod + high_sod,
                low_bucket_size * high_bucket_size,
                [
                    low_tws[0],
                    low_tws[1],
                    low_tws[2],
                    low_tws[3],
                    low_tws[4],
                    low_tws[5],
                    low_tws[6],
                    low_tws[7],
                    high_tws[0],
                    high_tws[1],
                    high_tws[2],
                    high_tws[3],
                    high_tws[4],
                    high_tws[5],
                    high_tws[6],
                    high_tws[7],
                ],
            )
        })
    })
}

/// Searches the eight substrings with width `bits` of a `feature`.
///
/// The target weights `tws` must be known as well.
pub fn search8(
    bits: u32,
    feature: u128,
    tws: [u32; 4],
    radius: u32,
) -> impl Iterator<Item = (usize, u32, usize, [u32; 8])> {
    const NPAIRS: u32 = 4;
    // Get the mask for the substring couples.
    let mask = (1u128 << (bits * NPAIRS)) - 1;
    // Split the `feature` into an array of substrings.
    let substrings = [feature & mask, feature >> (NPAIRS * bits)];

    let low_indices = search4(bits, substrings[0], [tws[0], tws[1]], radius);
    low_indices.flat_map(move |(low_index, low_sod, low_bucket_size, low_tws)| {
        let high_indices = search4(bits, substrings[1], [tws[2], tws[3]], radius - low_sod);
        high_indices.map(move |(high_index, high_sod, high_bucket_size, high_tws)| {
            (
                high_index * low_bucket_size + low_index,
                low_sod + high_sod,
                low_bucket_size * high_bucket_size,
                [
                    low_tws[0],
                    low_tws[1],
                    low_tws[2],
                    low_tws[3],
                    high_tws[0],
                    high_tws[1],
                    high_tws[2],
                    high_tws[3],
                ],
            )
        })
    })
}

/// Searches the four substrings with width `bits` of a `feature`.
///
/// The target weights `tws` must be known as well.
pub fn search4(
    bits: u32,
    feature: u128,
    tws: [u32; 2],
    radius: u32,
) -> impl Iterator<Item = (usize, u32, usize, [u32; 4])> {
    const NPAIRS: u32 = 2;
    // Get the mask for the substring couples.
    let mask = (1u128 << (bits * NPAIRS)) - 1;
    // Split the `feature` into an array of substrings.
    let substrings = [feature & mask, feature >> (NPAIRS * bits)];

    let low_indices = search2(bits, substrings[0], tws[0], radius);
    low_indices.flat_map(move |(low_index, low_sod, low_bucket_size, low_tws)| {
        let high_indices = search2(bits, substrings[1], tws[1], radius - low_sod);
        high_indices.map(move |(high_index, high_sod, high_bucket_size, high_tws)| {
            (
                high_index * low_bucket_size + low_index,
                low_sod + high_sod,
                low_bucket_size * high_bucket_size,
                [low_tws[0], low_tws[1], high_tws[0], high_tws[1]],
            )
        })
    })
}

/// Searches the two substrings with `bits` bits of a `feature`.
///
/// The target weight `tw` must be known as well.
pub fn search2(
    bits: u32,
    feature: u128,
    tw: u32,
    radius: u32,
) -> impl Iterator<Item = (usize, u32, usize, [u32; 2])> {
    // Get the number of ones in the search word.
    let sw = feature.count_ones();
    // Get the number of ones in the left half.
    let sl = ((feature >> bits) & ((1u128 << bits) - 1)).count_ones();

    let max = std::cmp::min(tw, bits);
    let min = tw - max;

    let (indices, bucket_size) = search(bits, sl, sw, tw, radius);
    indices.map(move |(index, sod)| {
        (
            index as usize,
            sod,
            bucket_size as usize,
            [tw - (index + min), index + min],
        )
    })
}

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
        let min_inflection = std::cmp::min(inflection1, inflection2);
        let max_inflection = std::cmp::max(inflection1, inflection2);
        let end = (radius + c) / 2;

        let down = start..min_inflection;
        let flat = min_inflection..=max_inflection;
        let up = max_inflection + 1..=end;

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

        // [58, 14] ([sl, sr])
        let (indices, size) = search_sort(64, 58, 72, 75, 10);
        assert_eq!(
            &indices,
            &[
                (55, 9),
                (56, 7),
                (57, 5),
                (58, 3),
                (59, 3),
                (60, 3),
                (61, 3),
                (62, 5),
                (63, 7),
                (64, 9)
            ]
        );
        assert_eq!(size, 54);

        // [58, 14] ([sl, sr])
        let (indices, size) = search_sort(64, 58, 72, 76, 10);
        assert_eq!(
            &indices,
            &[
                (55, 10),
                (56, 8),
                (57, 6),
                (58, 4),
                (59, 4),
                (60, 4),
                (61, 4),
                (62, 4),
                (63, 6),
                (64, 8)
            ]
        );
        assert_eq!(size, 53);

        // [58, 14] ([sl, sr])
        let (indices, size) = search_sort(64, 58, 72, 82, 10);
        assert_eq!(
            &indices,
            &[
                (58, 10),
                (59, 10),
                (60, 10),
                (61, 10),
                (62, 10),
                (63, 10),
                (64, 10)
            ]
        );
        assert_eq!(size, 47);

        // [58, 14] ([sl, sr])
        let (indices, size) = search_sort(64, 58, 72, 83, 10);
        assert_eq!(&indices, &[]);
        assert_eq!(size, 46);

        // [0, 2] ([sl, sr])
        let (indices, size) = search_sort(64, 0, 2, 2, 0);
        assert_eq!(&indices, &[(0, 0)]);
        assert_eq!(size, 3);
    }
}
