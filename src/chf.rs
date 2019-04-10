//! This module attempts to solve a single problem: iteratively computing the
//! substring hamming distances between a search feature and a target feature
//! for which we have the whole search feature and certain weights of the
//! target.
//!
//! At each particular node in the tree, we can get the canonical form of the
//! weights of all left substrings. What this means is that we keep splitting
//! the tree in half until we get `2^n` substrings. We then only encode the
//! left substring in each substring pair because at a given node of the tree
//! all of the parent substrings weights are constant, therefore each left
//! substring weight corresponds to a single right substring weight given by
//! the formula `|parent| - |left| = |right|`. This makes intuitive sense
//! the amount of one bits in each half of the number should be the same as
//! the number of bits in the whole number.
//!
//! So, if we have an actual 2 bit number, the canonical left-form of this
//! number is literally just the left bit masked out and the right set to
//! zero (`A0`). This is because the bit actually counts itself. In a one
//! bit number, the amount of bits set is equal to the number itself. If
//! we have a four bit number, this gets more complicated.
//!
//! Consider the situation where we have a four bit number `ABCD`. This number
//! has two substring configurations: `[A, B, C, D]` and `[AB, CD]`. The first
//! one is literally just the individual bits of the number. We will henceforth
//! call the canonical form Canonical Left Hamming Form (CLHF). The CLHF of
//! the first string is `A0C0` which can be obtained by masking every other bit
//! by doing `n & 0b1010`. The second string can be computed by summing the
//! CLHF shifted one to the right with the Canonical Right Hamming Form (CRHF)
//! and then masking the appropriate bits. This comes out to be
//! `(0A0C + 0B0D) & 0b1100`. This pattern continues for longer bit strings.
//!
//! One issue with the above algorithm is that as we descend the HWT, we only
//! have access to the parent substring weight and the left substring weight.
//! We would also ideally like to avoid converting it out of canonical form.
//! We will call the canonical form of the parent substring weights the
//! Canonical Hamming Form (CHF). We can compute the CRHF using the CLHF and
//! CHF in two basic SIMD In Register (SIR) operations:
//!
//! - Let `BITS` be the bits per substring.
//! - `let CRHF = CHF - (CLHF >> BITS)`
//!
//! This works because the parent substring weights will always be larger than
//! the left child substring weight, so we don't have to worry about borrowing.
//!
//! Now we have seen that it is possible to compute the CRHF from the CLHF.
//! We also know how to compute the CHF from an input feature. We can also
//! turn a CHF into a CRHF and CLHF through basic masking. Let us now define
//! the CHFn, where `n` is replaced with the level of the CHF. The complete
//! hamming weight of the number is CHF0, which means there is `2^n` substrings
//! which have been turned into their own weights, and `2^0 = 1`. The CHF1 is
//! therefore the weights of two substrings in-place. The CLHF doesn't make any
//! sense in the context of a single string, so we leave CLHF0 undefined. This
//! makes things simpler, as we will see.
//!
//! - Let `BITS` be the number of bits in the whole number.
//! - `CLHF1 = CHF1 & (((1 << (BITS / 2)) - 1) << (BITS / 2))`
//! - `CRHF1 = CHF1 & ((1 << (BITS / 2)) - 1)`
//! - `CRHF1 = CHF0 - CLHF1 >> (BITS / 2)`
//!
//! The masks and shifts are constant for any `n` and `BITS`. We can define
//! constant functions which compute these masks at compile time.
//!
//! This module contains routines for doing the above operations.

/// Computes the mask for a CLHF at `level`. Panics if level is not in the
/// range `[1, 7]`.
pub fn clhf_mask(level: u32) -> u128 {
    match level {
        7 => 0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA,
        1...6 => {
            let next = clhf_mask(level + 1);
            next >> (2 * (7 - level))
        }
        _ => panic!("hwt::chf::clhf_mask() must be passed a level in the range [1, 7]"),
    }
}

const CLHF_MASKS: [u128; 7] = [
    0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000,
    0xFFFF_FFFF_0000_0000_FFFF_FFFF_0000_0000,
    0xFFFF_0000_FFFF_0000_FFFF_0000_FFFF_0000,
    0xFF00_FF00_FF00_FF00_FF00_FF00_FF00_FF00,
    0xF0F0_F0F0_F0F0_F0F0_F0F0_F0F0_F0F0_F0F0,
    0xCCCC_CCCC_CCCC_CCCC_CCCC_CCCC_CCCC_CCCC,
    0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA,
];

const CRHF_MASKS: [u128; 7] = [
    0x0000_0000_0000_0000_FFFF_FFFF_FFFF_FFFF,
    0x0000_0000_FFFF_FFFF_0000_0000_FFFF_FFFF,
    0x0000_FFFF_0000_FFFF_0000_FFFF_0000_FFFF,
    0x00FF_00FF_00FF_00FF_00FF_00FF_00FF_00FF,
    0x0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F,
    0x3333_3333_3333_3333_3333_3333_3333_3333,
    0x5555_5555_5555_5555_5555_5555_5555_5555,
];

/// Converts a CHF<n> to a CLHF<n>
pub fn chf_to_clhf(chf: u128, n: usize) -> u128 {
    match n {
        0 => panic!("hwt::chf::chf_to_clhf() can't create a CLHF<0>"),
        1...7 => chf & CLHF_MASKS[n - 1],
        n => panic!("hwt::chf::chf_to_clhf() can't create a CLHF<{}>", n),
    }
}

/// Converts a CHF<n> to a CRHF<n>
pub fn chf_to_crhf(chf: u128, n: usize) -> u128 {
    match n {
        0 => panic!("hwt::chf::chf_to_crhf() can't create a CRHF<0>"),
        1...7 => chf & CRHF_MASKS[n - 1],
        n => panic!("hwt::chf::chf_to_crhf() can't create a CRHF<{}>", n),
    }
}

/// Computes CRHF<n+1> from CHF<n> and CLHF<n+1>.
pub fn compute_crhf(chf: u128, clhf: u128, n: usize) -> u128 {
    match n {
        0...6 => chf - (clhf >> (1 << (6 - n))),
        _ => panic!("hwt::chf::chf_to_crhf() can't create a CRHF<{}>", n),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chf_to_clhf() {
        assert_eq!(
            chf_to_clhf(0xDEAD_BEEF_DEAD_BEEF_DEAD_BEEF_DEAD_BEEF, 3),
            0xDEAD_0000_DEAD_0000_DEAD_0000_DEAD_0000
        );
    }

    #[test]
    fn test_chf_to_crhf() {
        assert_eq!(
            chf_to_crhf(0xDEAD_BEEF_DEAD_BEEF_DEAD_BEEF_DEAD_BEEF, 3),
            0x0000_BEEF_0000_BEEF_0000_BEEF_0000_BEEF
        );
    }

    #[test]
    fn test_compute_crhf() {
        assert_eq!(
            compute_crhf(
                0x0000_0003_0000_0005_0000_0004_0000_0001,
                0x0001_0000_0001_0000_0001_0000_0001_0000,
                2
            ),
            0x0000_0002_0000_0004_0000_0003_0000_0000
        );
    }
}
