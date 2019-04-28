use itertools::Itertools;
use swar::*;

/// Gets all the possible offsets in a feature that maintain a particular
/// radius at max.
///
/// - `sp` - Search parent CHF<6>
/// - `sc` - Search child CHF<7>
/// - `tp` - Target parent CHF<6>
///
/// Returns an iterator over the (tc, sod) target children
/// and sum of distance pairs.
pub fn search_radius128(
    bits: u32,
    sp: Bits2<u128>,
    sc: Bits1<u128>,
    tp: Bits2<u128>,
    radius: u32,
) -> impl Iterator<Item = (Bits1<u128>, u32)> {
    let (lsp, rsp) = sp.halve();
    let (lsc, rsc) = sc.halve();
    let (ltp, rtp) = tp.halve();

    Box::new(
        search_radius64(bits, lsp, lsc, ltp, radius).flat_map(move |(ltc, lsod)| {
            search_radius64(bits, rsp, rsc, rtp, radius - lsod)
                .map(move |(rtc, rsod)| (Bits1::union(ltc, rtc), lsod + rsod))
        }),
    )
}

/// Gets all the possible offsets in a feature that maintain a particular
/// radius at max.
///
/// - `sp` - Search parent CHF<5>
/// - `sc` - Search child CHF<6>
/// - `tp` - Target parent CHF<5>
///
/// Returns an iterator over the (tc, sod) target children
/// and sum of distance pairs.
pub fn search_radius64(
    bits: u32,
    sp: Bits4<u128>,
    sc: Bits2<u128>,
    tp: Bits4<u128>,
    radius: u32,
) -> impl Iterator<Item = (Bits2<u128>, u32)> {
    let (lsp, rsp) = sp.halve();
    let (lsc, rsc) = sc.halve();
    let (ltp, rtp) = tp.halve();

    Box::new(
        search_radius32(bits, lsp, lsc, ltp, radius).flat_map(move |(ltc, lsod)| {
            search_radius32(bits, rsp, rsc, rtp, radius - lsod)
                .map(move |(rtc, rsod)| (Bits2::union(ltc, rtc), lsod + rsod))
        }),
    )
}

/// Gets all the possible offsets in a feature that maintain a particular
/// radius at max.
///
/// - `sp` - Search parent CHF<4>
/// - `sc` - Search child CHF<5>
/// - `tp` - Target parent CHF<4>
///
/// Returns an iterator over the (tc, sod) target children
/// and sum of distance pairs.
pub fn search_radius32(
    bits: u32,
    sp: Bits8<u128>,
    sc: Bits4<u128>,
    tp: Bits8<u128>,
    radius: u32,
) -> impl Iterator<Item = (Bits4<u128>, u32)> {
    let (lsp, rsp) = sp.halve();
    let (lsc, rsc) = sc.halve();
    let (ltp, rtp) = tp.halve();

    Box::new(
        search_radius16(bits, lsp, lsc, ltp, radius).flat_map(move |(ltc, lsod)| {
            search_radius16(bits, rsp, rsc, rtp, radius - lsod)
                .map(move |(rtc, rsod)| (Bits4::union(ltc, rtc), lsod + rsod))
        }),
    )
}

/// Gets all the possible offsets in a feature that maintain a particular
/// radius at max.
///
/// - `sp` - Search parent CHF<3>
/// - `sc` - Search child CHF<4>
/// - `tp` - Target parent CHF<3>
///
/// Returns an iterator over the (tc, sod) target children
/// and sum of distance pairs.
pub fn search_radius16(
    bits: u32,
    sp: Bits16<u128>,
    sc: Bits8<u128>,
    tp: Bits16<u128>,
    radius: u32,
) -> impl Iterator<Item = (Bits8<u128>, u32)> {
    let (lsp, rsp) = sp.halve();
    let (lsc, rsc) = sc.halve();
    let (ltp, rtp) = tp.halve();

    search_radius8(bits, lsp, lsc, ltp, radius).flat_map(move |(ltc, lsod)| {
        search_radius8(bits, rsp, rsc, rtp, radius - lsod)
            .map(move |(rtc, rsod)| (Bits8::union(ltc, rtc), lsod + rsod))
    })
}

/// Gets all the possible offsets in a feature that maintain a particular
/// radius at max.
///
/// - `sp` - Search parent CHF<2>
/// - `sc` - Search child CHF<3>
/// - `tp` - Target parent CHF<2>
///
/// Returns an iterator over the (tc, sod) target children
/// and sum of distance pairs.
pub fn search_radius8(
    bits: u32,
    sp: Bits32<u128>,
    sc: Bits16<u128>,
    tp: Bits32<u128>,
    radius: u32,
) -> impl Iterator<Item = (Bits16<u128>, u32)> {
    let (lsp, rsp) = sp.halve();
    let (lsc, rsc) = sc.halve();
    let (ltp, rtp) = tp.halve();

    Box::new(
        search_radius4(bits, lsp, lsc, ltp, radius).flat_map(move |(ltc, lsod)| {
            search_radius4(bits, rsp, rsc, rtp, radius - lsod)
                .map(move |(rtc, rsod)| (Bits16::union(ltc, rtc), lsod + rsod))
        }),
    )
}

/// Gets all the possible offsets in a feature that maintain a particular
/// radius at max.
///
/// - `sp` - Search parent CHF<1>
/// - `sc` - Search child CHF<2>
/// - `tp` - Target parent CHF<1>
///
/// Returns an iterator over the (tc, sod) target children
/// and sum of distance pairs.
pub fn search_radius4(
    bits: u32,
    sp: Bits64<u128>,
    sc: Bits32<u128>,
    tp: Bits64<u128>,
    radius: u32,
) -> impl Iterator<Item = (Bits32<u128>, u32)> {
    let (lsp, rsp) = sp.halve();
    let (lsc, rsc) = sc.halve();
    let (ltp, rtp) = tp.halve();

    search_radius2(bits, lsp, lsc, ltp, radius).flat_map(move |(ltc, lsod)| {
        search_radius2(bits, rsp, rsc, rtp, radius - lsod)
            .map(move |(rtc, rsod)| (Bits32::union(ltc, rtc), lsod + rsod))
    })
}

/// Gets all the possible offsets in a feature that maintain a particular
/// radius at max.
///
/// - `sp` - Search parent CHF<0>
/// - `sc` - Search child CHF<1>
/// - `tp` - Target parent CHF<0>
///
/// Returns an iterator over the (tc, sod) target children
/// and sum of distance pairs.
pub fn search_radius2(
    bits: u32,
    sp: Bits128<u128>,
    sc: Bits64<u128>,
    tp: Bits128<u128>,
    radius: u32,
) -> impl Iterator<Item = (Bits64<u128>, u32)> {
    // Get the number of ones in the search word.
    let sw = sp.count_ones();
    // Get the number of ones in the left half.
    let sl = (sc >> 64).count_ones();
    // Get the number of ones in the target word.
    let tw = tp.count_ones();

    search_radius(bits, sl, sw, tw, radius)
        .map(|([tl, tr], sod)| (Bits64(((1 << tl) - 1) << 64 | ((1 << tr) - 1)), sod))
}

/// Iterator over the indices that fall within a radius of a number.
///
/// - `bits` - The number of bits that make up the bit substring `sl`
///     comes from.
/// - `sl` - The weight of the left half of the search number.
/// - `sw` - The weight of the whole search number.
/// - `tw` - The weight of the whole target number.
/// - `radius` - The maximum possible sum of distances (sod) of matches.
///
/// Returns the iterator over (tl, tr, sod).
pub fn search_radius(
    bits: u32,
    sl: u32,
    sw: u32,
    tw: u32,
    radius: u32,
) -> impl Iterator<Item = ([u32; 2], u32)> {
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
            [tl as u32, (tw - tl) as u32],
            ((tl - sl).abs() + ((tw - tl) - (sw - sl)).abs()) as u32,
        )
    };

    let bottom_distance = (tw - sw).abs();

    // Check if we intersect.
    if bottom_distance <= radius {
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
        flat.chain(down.interleave(up)).filter(filter).map(map)
    } else {
        // Create fake iterators to satisfy the type system.
        let down = 0..0;
        let flat = 0..=-1;
        let up = 0..=-1;

        // Also perform the same operations over here.
        flat.chain(down.interleave(up)).filter(filter).map(map)
    }
}
