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

use swar::*;

/// Computes CRHF<1> from CHF<0> and CLHF<1>.
pub fn crhf1(chf: u128, clhf: Bits64<u128>) -> Bits64<u128> {
    Bits64(chf - (clhf.0 >> 64))
}

/// Computes CRHF<2> from CHF<1> and CLHF<2>.
pub fn crhf2(chf: Bits64<u128>, clhf: Bits32<u128>) -> Bits32<u128> {
    Bits32(chf.0 - (clhf.0 >> 32))
}

/// Computes CRHF<3> from CHF<2> and CLHF<3>.
pub fn crhf3(chf: Bits32<u128>, clhf: Bits16<u128>) -> Bits16<u128> {
    Bits16(chf.0 - (clhf.0 >> 16))
}

/// Computes CRHF<4> from CHF<3> and CLHF<4>.
pub fn crhf4(chf: Bits16<u128>, clhf: Bits8<u128>) -> Bits8<u128> {
    Bits8(chf.0 - (clhf.0 >> 8))
}

/// Computes CRHF<5> from CHF<4> and CLHF<5>.
pub fn crhf5(chf: Bits8<u128>, clhf: Bits4<u128>) -> Bits4<u128> {
    Bits4(chf.0 - (clhf.0 >> 4))
}

/// Computes CRHF<6> from CHF<5> and CLHF<6>.
pub fn crhf6(chf: Bits4<u128>, clhf: Bits2<u128>) -> Bits2<u128> {
    Bits2(chf.0 - (clhf.0 >> 2))
}

/// Computes CRHF<7> from CHF<6> and CLHF<7>.
pub fn crhf7(chf: Bits2<u128>, clhf: Bits1<u128>) -> Bits1<u128> {
    Bits1(chf.0 - (clhf.0 >> 1))
}
