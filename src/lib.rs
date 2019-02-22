//! Here is how we would like to think about a number visually, in
//! terms of a binary tree of its substring hamming weights:
//!
//! ```no_build
//!        5
//!    3       2
//!  2   1   1   1
//! 1 1 0 1 1 0 0 1
//! ```
//!
//! Let `B` be the log2 of the width of the number. In this case `B = 3`,
//! since `2^3 = 8`.
//!
//! Let `L` be the level of the hamming tree. The hamming
//! weight of the whole number is the root and is `L = 0`.
//!
//! Let `N` be the index at the level `L` of the substring weight in question.
//!
//! Let `W` be a weight of the substring `N` at level `L`.
//!
//! Let `MAX` be the side max of the hamming tree. `MAX = min(W, 2^(B - L - 1))`.
//! This is the maximum number of ones that either side of a substring can
//! have.
//!
//! Let `MIN` be the side min of the hamming tree. `MIN = W - MAX`.
//! This is the minimum number of ones that either side of a substring can
//! have.
//!
//! Every time we encounter a weight `W` in the tree then the next two
//! substrings can vary from `[MIN, MAX]` to `[MAX, MIN]` for a total of
//! `A + 1` possibilities. Therefore we can also view the tree like this:
//!
//! ```no_build
//!        5                   [1-4]                   1
//!    3       2         [1-2]       [1-2]         2       1
//!  2   1   1   1    [1-1] [0-1] [0-1] [0-1]    0   0   1   0
//! 1 1 0 1 1 0 0 1
//! ```
//!
//! On the left we have the actual tree. In the middle we have the
//! possible values for the left branch. On the right we have the index
//! of the left branch chosen, which is calculated by subtracting the left
//! substring weight by `MIN`.
//!
//! To compute the index for `L` we must iteratively multiply an accumulator
//! by `MAX - MIN + 1` of the current substring `N`, add the substring's index
//! from the tree, then shift the number over by the substring width to get
//! `N + 1`.
//!
//! To do the reverse, we must mod the accumulator by the multiplication of
//! all lower substring `MAX - MIN + 1` to get the index of that substring
//! and then divide by the `MAX - MIN + 1` of the current substring.
//! Do this iteratively to produce all weights for a given index.
//! We should avoid computing the weights from the index more than once
//! per operation if possible because it is costly due to modulo and division.

mod indices;

use indices::*;

const HIGH: u32 = 0x8000_0000;

pub struct Hwt {
    /// The u32 points to a location in the internals array that is the
    /// start of a slice of internal or leaf node indices. If an internal
    /// has a high bit set to `1` then it is a leaf node.
    internals: Vec<u32>,
    count: usize,
}

impl Hwt {
    /// Makes an empty `Hwt`.
    ///
    /// ```
    /// # use hwt::Hwt;
    /// let hwt = Hwt::new();
    /// assert!(hwt.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the number of entries in the `Hwt`.
    ///
    /// ```
    /// # use hwt::Hwt;
    /// let hwt = Hwt::new();
    /// assert_eq!(hwt.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.count
    }

    /// Checks if the `Hwt` is empty.
    ///
    /// ```
    /// # use hwt::Hwt;
    /// let hwt = Hwt::new();
    /// assert!(hwt.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Inserts an item ID to the `Hwt`.
    ///
    /// The most significant bit must not be set on the `item`.
    ///
    /// Returns `Some(t)` if item `t` was replaced by `item`.
    ///
    /// ```
    /// # use hwt::Hwt;
    /// let hwt = Hwt::new();
    /// assert!(hwt.is_empty());
    /// ```
    pub fn insert(&mut self, feature: u128, item: u32) -> Option<u32> {
        assert_eq!(item & HIGH, 0);
        unimplemented!()
    }
}

impl Default for Hwt {
    fn default() -> Self {
        // The number of child nodes of the root is determined by the different
        // possible hamming weights. The maximum hamming weight is the number
        // of bits and the minimum is 0, so this means that there are
        // `NBits + 1` child nodes.
        Self {
            internals: vec![0; 129],
            count: 0,
        }
    }
}
