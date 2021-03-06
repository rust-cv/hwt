use swar::*;

/// Compute the indices for a 128-bit integer,
/// along with the overall `MAX - MIN`.
///
/// It is possible for the last index to have a bucket size that can only fit
/// in a `u128`.
#[inline(always)]
pub fn indices128(v: u128) -> [u128; 8] {
    let v7 = Bits1(v);
    let v6 = v7.pack_ones();
    let v5 = v6.pack_ones();
    let v4 = v5.pack_ones();
    let v3 = v4.pack_ones();
    let v2 = v3.pack_ones();
    let v1 = v2.pack_ones();
    let v0 = v1.pack_ones();
    [v0.0, v1.0, v2.0, v3.0, v4.0, v5.0, v6.0, v7.0]
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
