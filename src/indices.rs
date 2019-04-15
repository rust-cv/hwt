use std::cmp::min;
use swar::*;

/// Compute the indices for a 128-bit integer,
/// along with the overall `MAX - MIN`.
///
/// It is possible for the last index to have a bucket size that can only fit
/// in a `u128`.
pub fn indices128(v: u128) -> [u128; 8] {
    let v7 = Bits1(v);
    let v6 = v7.sum_weight2();
    let v5 = v6.sum_weight2();
    let v4 = v5.sum_weight2();
    let v3 = v4.sum_weight2();
    let v2 = v3.sum_weight2();
    let v1 = v2.sum_weight2();
    let v0 = v1.sum_weight2();
    [v0.0, v1.0, v2.0, v3.0, v4.0, v5.0, v6.0, v7.0]
}

/// Compute the indices for a 32-bit integer,
/// along with the overall `MAX - MIN`.
pub fn indices64(v: u64) -> ([usize; 6], [usize; 6]) {
    const NBITS: u32 = 64;
    const HBITS: u32 = NBITS / 2;
    let ones = v.count_ones();
    let left = (v >> HBITS).count_ones();
    let lmax = min(ones, HBITS);
    let lmin = ones - lmax;
    let index = (left - lmin) as usize;

    // Get the indices and the `MAX - MIN` for the lower levels.
    let halves = [indices32(v as u32), indices32((v >> HBITS) as u32)];
    (
        [
            index,
            halves[0].0[0] + halves[1].0[0] * halves[0].1[0],
            halves[0].0[1] + halves[1].0[1] * halves[0].1[1],
            halves[0].0[2] + halves[1].0[2] * halves[0].1[2],
            halves[0].0[3] + halves[1].0[3] * halves[0].1[3],
            halves[0].0[4] + halves[1].0[4] * halves[0].1[4],
        ],
        [
            (lmax - lmin + 1) as usize,
            halves[0].1[0] * halves[1].1[0],
            halves[0].1[1] * halves[1].1[1],
            halves[0].1[2] * halves[1].1[2],
            halves[0].1[3] * halves[1].1[3],
            halves[0].1[4] * halves[1].1[4],
        ],
    )
}

/// Compute the indices for a 32-bit integer,
/// along with the overall `MAX - MIN`.
pub fn indices32(v: u32) -> ([usize; 5], [usize; 5]) {
    const NBITS: u32 = 32;
    const HBITS: u32 = NBITS / 2;
    let ones = v.count_ones();
    let left = (v >> HBITS).count_ones();
    let lmax = min(ones, HBITS);
    let lmin = ones - lmax;
    let index = (left - lmin) as usize;

    // Get the indices and the `MAX - MIN` for the lower levels.
    let halves = [indices16(v as u16), indices16((v >> HBITS) as u16)];
    (
        [
            index,
            halves[0].0[0] + halves[1].0[0] * halves[0].1[0],
            halves[0].0[1] + halves[1].0[1] * halves[0].1[1],
            halves[0].0[2] + halves[1].0[2] * halves[0].1[2],
            halves[0].0[3] + halves[1].0[3] * halves[0].1[3],
        ],
        [
            (lmax - lmin + 1) as usize,
            halves[0].1[0] * halves[1].1[0],
            halves[0].1[1] * halves[1].1[1],
            halves[0].1[2] * halves[1].1[2],
            halves[0].1[3] * halves[1].1[3],
        ],
    )
}

/// Compute the indices for a 16-bit integer,
/// along with the overall `MAX - MIN`.
pub fn indices16(v: u16) -> ([usize; 4], [usize; 4]) {
    const NBITS: u32 = 16;
    const HBITS: u32 = NBITS / 2;
    let ones = v.count_ones();
    let left = (v >> HBITS).count_ones();
    let lmax = min(ones, HBITS);
    let lmin = ones - lmax;
    let index = (left - lmin) as usize;

    // Get the indices and the `MAX - MIN` for the lower levels.
    let halves = [indices8(v as u8), indices8((v >> HBITS) as u8)];
    (
        [
            index,
            halves[0].0[0] + halves[1].0[0] * halves[0].1[0],
            halves[0].0[1] + halves[1].0[1] * halves[0].1[1],
            halves[0].0[2] + halves[1].0[2] * halves[0].1[2],
        ],
        [
            (lmax - lmin + 1) as usize,
            halves[0].1[0] * halves[1].1[0],
            halves[0].1[1] * halves[1].1[1],
            halves[0].1[2] * halves[1].1[2],
        ],
    )
}

/// Compute the indices for an 8-bit integer,
/// along with the overall `MAX - MIN`.
#[inline(always)]
pub fn indices8(v: u8) -> ([usize; 3], [usize; 3]) {
    const NBITS: u32 = 8;
    const HBITS: u32 = NBITS / 2;
    const MASK: u8 = (1 << HBITS) - 1;
    let ones = v.count_ones();
    let left = (v >> HBITS).count_ones();
    let lmax = min(ones, HBITS);
    let lmin = ones - lmax;
    let index = (left - lmin) as usize;

    // Get the indices and the `MAX - MIN` for the lower levels.
    let halves = [indices4(v & MASK), indices4(v >> HBITS)];
    (
        [
            index,
            halves[0].0[0] + halves[1].0[0] * halves[0].1[0],
            halves[0].0[1] + halves[1].0[1] * halves[0].1[1],
        ],
        [
            (lmax - lmin + 1) as usize,
            halves[0].1[0] * halves[1].1[0],
            halves[0].1[1] * halves[1].1[1],
        ],
    )
}

/// Compute the indices for a 4-bit integer,
/// along with the overall `MAX - MIN`.
#[inline(always)]
pub fn indices4(v: u8) -> ([usize; 2], [usize; 2]) {
    const NBITS: u32 = 4;
    const HBITS: u32 = NBITS / 2;
    const MASK: u8 = (1 << HBITS) - 1;
    let ones = v.count_ones();
    let left = (v >> HBITS).count_ones();
    // Get the indices and the `MAX - MIN` for the lower level.
    let halves = [indices2(v & MASK), indices2(v >> HBITS)];
    let lmax = min(ones, HBITS);
    let lmin = ones - lmax;
    (
        [
            (left - lmin) as usize,
            halves[0].0[0] + halves[1].0[0] * halves[0].1[0],
        ],
        [(lmax - lmin + 1) as usize, halves[0].1[0] * halves[1].1[0]],
    )
}

/// Compute the indices for a 2-bit integer,
/// along with the overall `MAX - MIN`.
#[inline(always)]
pub fn indices2(v: u8) -> ([usize; 1], [usize; 1]) {
    let different = (v >> 1) ^ (v & 0b1);
    let possibilities = different + 1;
    let choice = different & (v >> 1);
    ([choice as usize], [possibilities as usize])
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_indices128() {
        assert_eq!(
            indices128(0x0000_0000_0000_0000_0000_0000_0000_0000),
            [0, 0, 0, 0, 0, 0, 0, 0]
        );

        assert_eq!(
            indices128(0x0000_0040_0000_0000_0000_0000_0000_0000),
            [1, 1 << 64, 1 << (32 * 3), 1 << (16 * 6), 1 << (8 * 12), 1 << (4 * 25), 1 << (2 * 51), 1 << 102]
        );

        assert_eq!(
            indices128(0x0000_0000_0000_0000_FFFF_FFFF_FFFF_FFFF),
            [0, 0, 0, 0, 0, 0, 0, 0]
        );

        assert_eq!(
            indices128(0xFFFF_FFFF_FFFF_FFF0_0000_0000_0000_000F),
            [0, 0, 0, 0, 0, 0, 0, 0]
        );

        assert_eq!(
            indices128(0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF),
            [0, 0, 0, 0, 0, 0, 0, 0]
        );
    }
}
