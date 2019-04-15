use swar::*;
use crate::search::*;
use std::iter::once;

/// Gets all the possible offsets in a feature that maintain a particular
/// radius at max.
/// 
/// - `sp` - Search parent CHF<1>
/// - `sc` - Search child CHF<2>
/// - `tp` - Target parent CHF<1>
/// 
/// Returns an iterator over the `tc` target children at that radius.
pub fn search_exact4(
    sp: Bits64<u128>,
    sc: Bits32<u128>,
    tp: Bits64<u128>,
    radius: u32,
) -> impl Iterator<Item = Bits64<u128>> {
    let (usp, lsp) = (sp.0 >> 64, sp.0 & 0x);
    let (usc, lsc) = sc.split();
    let (utp, ltp) = tp.split();

    search_radius_raw2(usp.0, usc.0, utp.0, radius).flat_map(|(, sod)| {
        // The new radius to search for since we found a radius sod.
        let radius = radius - sod;
        search_exact_raw2(lsp.0, lsc.0, ltp.0, radius).map(|lower| {
            
        })
    })
}

/// Gets all the possible offsets in a feature that maintain a particular
/// radius at max.
/// 
/// - `sp` - Search parent CHF<0>
/// - `sc` - Search child CHF<1>
/// - `tp` - Target parent CHF<0>
/// 
/// Returns an iterator over the `tc` target children at that radius.
pub fn search_exact2(
    sp: Bits128<u128>,
    sc: Bits64<u128>,
    tp: Bits128<u128>,
    radius: u32,
) -> impl Iterator<Item = Bits64<u128>> {
    // Get the number of ones in the search word.
    let sw = sp.0 as u32;
    // Get the number of ones in the left half.
    let sl = (sc.0 >> 64) as u32;
    // Get the number of ones in the target word.
    let tw = tp.0 as u32;

    search_exact(64, sl, sw, tw, radius).map(|(tl, tr)| Bits64((tl as u128) << 64 | tr as u128))
}

/// Iterator over the indices that fall within a radius of a number.
///
/// - `bits` - The number of bits the that make up the bit substring `sl`
///     comes from.
/// - `sl` - The weight of the left half of the search number.
/// - `sw` - The weight of the whole search number.
/// - `tw` - The weight of the whole target number.
/// - `radius` - The exact sum of distances (sod) of matches.
///
/// Returns the iterator over (tl, tr).
pub fn search_exact(
    bits: u32,
    sl: u32,
    sw: u32,
    tw: u32,
    radius: u32,
) -> impl Iterator<Item = [u32; 2]> {
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

    let bottom_distance = ((radius + c) / 2 - sl).abs() + (tw - (radius + c) / 2 - sw + sl).abs();

    let map = |tl| [tl as u32, (tw - tl) as u32];

    if bottom_distance == radius {
        // We intersect at the flat bottom, so get the inflection points
        // and use them to create the flat range.
        let inflection1 = sl;
        let inflection2 = sl - sw + tw;
        let min_inflection = std::cmp::min(inflection1, inflection2);
        let max_inflection = std::cmp::max(inflection1, inflection2);
        either::Left((min_inflection..=max_inflection).map(map))
    } else if bottom_distance < radius {
        // We intersect at precisely two locations.
        let start = (-radius + c + 1) / 2;
        let end = (radius + c) / 2;

        // We interleave `down` and `up` so that the resulting iterator always
        // goes in increasing `SOD` order. `flat` is always the best matches.
        if start == end {
            either::Left((start..=end).map(map))
        } else {
            either::Right(std::iter::once(start).chain(std::iter::once(end)).map(map))
        }
    } else {
        // Create fake iterators to satisfy the type system.
        let flat = 0..=-1;

        // Also perform the same operations over here.
        either::Left(flat.map(map))
    }
}