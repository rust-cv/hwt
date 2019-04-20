use swar::*;

/// Compute the indices for a 128-bit integer,
/// along with the overall `MAX - MIN`.
///
/// It is possible for the last index to have a bucket size that can only fit
/// in a `u128`.
#[inline(always)]
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

/// Finds the minimum distance at the given level.
#[inline(always)]
pub fn index_distance(target: u128, indices: &[u128; 8], level: u8) -> u32 {
    match level {
        0 => Bits128(target)
            .hwd(Bits128(indices[level as usize]))
            .sum_weight() as u32,
        1 => Bits64(target)
            .hwd(Bits64(indices[level as usize]))
            .sum_weight() as u32,
        2 => Bits32(target)
            .hwd(Bits32(indices[level as usize]))
            .sum_weight() as u32,
        3 => Bits16(target)
            .hwd(Bits16(indices[level as usize]))
            .sum_weight() as u32,
        4 => Bits8(target)
            .hwd(Bits8(indices[level as usize]))
            .sum_weight() as u32,
        5 => Bits4(target)
            .hwd(Bits4(indices[level as usize]))
            .sum_weight() as u32,
        6 => Bits2(target)
            .hwd(Bits2(indices[level as usize]))
            .sum_weight() as u32,
        7 => Bits1(target)
            .hwd(Bits1(indices[level as usize]))
            .sum_weight() as u32,
        _ => unreachable!(),
    }
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
            [
                1,
                1 << 64,
                1 << (32 * 3),
                1 << (16 * 6),
                1 << (8 * 12),
                1 << (4 * 25),
                1 << (2 * 51),
                1 << 102
            ]
        );
    }
}
