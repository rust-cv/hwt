use std::cmp::min;

/// Compute the indices for a 128-bit integer,
/// along with the overall `MAX - MIN`.
///
/// It is possible for the last index to have a bucket size that can only fit
/// in a `u128`.
pub fn indices128(v: u128) -> [usize; 7] {
    const NBITS: u32 = 128;
    const HBITS: u32 = NBITS / 2;
    let ones = v.count_ones();
    let left = (v >> HBITS).count_ones();
    let lmax = min(ones, HBITS);
    let lmin = ones - lmax;
    let index = (left - lmin) as usize;

    // Get the indices and the `MAX - MIN` for the lower levels.
    let halves = [indices64(v as u64), indices64((v >> HBITS) as u64)];
    [
        index,
        halves[0].0[0] + halves[1].0[0] * halves[0].1[0],
        halves[0].0[1] + halves[1].0[1] * halves[0].1[1],
        halves[0].0[2] + halves[1].0[2] * halves[0].1[2],
        halves[0].0[3] + halves[1].0[3] * halves[0].1[3],
        halves[0].0[4] + halves[1].0[4] * halves[0].1[4],
        halves[0].0[5] + halves[1].0[5] * halves[0].1[5],
    ]
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
    fn test_indices2() {
        assert_eq!(indices2(0b00), ([0], [1]));
        assert_eq!(indices2(0b01), ([0], [2]));
        assert_eq!(indices2(0b10), ([1], [2]));
        assert_eq!(indices2(0b11), ([0], [1]));
    }

    #[test]
    fn test_indices4() {
        assert_eq!(indices4(0b00_00), ([0, 0], [1, 1]));
        assert_eq!(indices4(0b00_10), ([0, 1], [2, 2]));
        assert_eq!(indices4(0b10_00), ([1, 1], [2, 2]));
        assert_eq!(indices4(0b11_10), ([1, 1], [2, 2]));
        assert_eq!(indices4(0b11_11), ([0, 0], [1, 1]));
    }

    #[test]
    fn test_indices8() {
        assert_eq!(indices8(0b00_00_00_00), ([0, 0, 0], [1, 1, 1]));
        assert_eq!(indices8(0b00_00_01_01), ([0, 1, 0], [3, 3, 4]));
        assert_eq!(indices8(0b00_00_11_11), ([0, 0, 0], [5, 1, 1]));
        assert_eq!(indices8(0b00_01_11_11), ([0, 0, 0], [4, 2, 2]));
        assert_eq!(indices8(0b01_00_11_11), ([0, 1, 0], [4, 2, 2]));
        assert_eq!(indices8(0b11_11_11_11), ([0, 0, 0], [1, 1, 1]));
    }

    #[test]
    fn test_indices16() {
        assert_eq!(
            indices16(0b00_00_00_00_00_00_00_00),
            ([0, 0, 0, 0], [1, 1, 1, 1])
        );

        assert_eq!(
            indices16(0b00_00_00_00_00_00_00_01),
            ([0, 0, 0, 0], [2, 2, 2, 2])
        );

        assert_eq!(
            indices16(0b00_00_00_01_00_00_00_00),
            ([1, 0, 0, 0], [2, 2, 2, 2])
        );

        assert_eq!(
            indices16(0b10_00_00_00_00_00_00_00),
            ([1, 1, 1, 1], [2, 2, 2, 2])
        );

        assert_eq!(
            indices16(0b11_11_00_00_11_00_00_00),
            ([4, 4 * 3 + 2, 2, 0], [7, 15, 3, 1])
        );

        assert_eq!(
            indices16(0b11_11_11_11_11_11_11_11),
            ([0, 0, 0, 0], [1, 1, 1, 1])
        );
    }

    #[test]
    fn test_indices128() {
        assert_eq!(
            indices128(0x0000_0000_0000_0000_0000_0000_0000_0000),
            [0, 0, 0, 0, 0, 0, 0]
        );

        assert_eq!(
            indices128(0x0000_0040_0000_0000_0000_0000_0000_0000),
            [1, 1, 0, 0, 1, 1, 0]
        );

        assert_eq!(
            indices128(0x0000_0000_0000_0000_FFFF_FFFF_FFFF_FFFF),
            [0, 0, 0, 0, 0, 0, 0]
        );

        assert_eq!(
            indices128(0xFFFF_FFFF_FFFF_FFF0_0000_0000_0000_000F),
            [60, 5 * 4, 5 * 4, 5 * 4, 5 * 4, 0, 0]
        );

        assert_eq!(
            indices128(0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF),
            [0, 0, 0, 0, 0, 0, 0]
        );
    }
}
