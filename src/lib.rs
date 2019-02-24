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

pub mod indices;

use indices::*;
use std::iter::repeat;

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
    /// let mut hwt = Hwt::new();
    /// hwt.insert(0b101, 0, |_| 0b010);
    /// assert_eq!(hwt.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.count
    }

    /// Checks if the `Hwt` is empty.
    ///
    /// ```
    /// # use hwt::Hwt;
    /// let mut hwt = Hwt::new();
    /// assert!(hwt.is_empty());
    /// hwt.insert(0b101, 0, |_| 0b010);
    /// assert!(!hwt.is_empty());
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
    /// let mut hwt = Hwt::new();
    /// hwt.insert(0b101, 0, |_| 0b010);
    /// hwt.insert(0b010, 1, |_| 0b101);
    /// assert_eq!(hwt.len(), 2);
    /// ```
    pub fn insert<F>(&mut self, feature: u128, item: u32, mut lookup: F) -> Option<u32>
    where
        F: FnMut(u32) -> u128,
    {
        assert_eq!(item & HIGH, 0);
        // Compute the indices of the buckets and the sizes of the buckets
        // for each layer of the tree.
        let (indices, sizes) = indices128(feature);
        // The first index in the tree is actually the overall weight of
        // the whole number.
        let weight = feature.count_ones() as usize;
        let mut node = weight;
        for i in 0..7 {
            match self.internals[node] {
                0 => {
                    self.internals[node] = item | HIGH;
                    self.count += 1;
                    return None;
                }
                internal if internal & HIGH == 0 => {
                    // Go to the next node.
                    node = internal as usize + indices[i];
                }
                leaf => {
                    // Check if the leaf is the same as this `item`.
                    if leaf & !HIGH == item {
                        // Replace it.
                        self.internals[node] = item | HIGH;
                        return Some(leaf & !HIGH);
                    }
                    // Get the leaf's indices. The size of any table we care
                    // about is the same as this `item`.
                    let (leaf_indices, _) = indices128(lookup(leaf & !HIGH));
                    // Iterate and make more child nodes until the `item` and
                    // the leaf differ.
                    for i in i..7 {
                        // Allocate the space for the next bucket.
                        // This will always be the same between both items.
                        let location = self.internals.len() as u32;
                        self.internals.extend(repeat(0).take(sizes[i]));
                        // Ensure the bucket index hasn't gotten larger than
                        // the max.
                        // TODO: Probably should be fixed by having a separate
                        // lookup table. This shouldn't be hit except in sets
                        // with hundreds of millions of items.
                        assert!(location & HIGH == 0);
                        // Add the bucket to this parent node.
                        self.internals[node] = location;
                        // Check if the indices are different.
                        if leaf_indices[i] != indices[i] {
                            // If they finally differ, we can add them to
                            // different spots and return.
                            self.internals[location as usize + leaf_indices[i]] = leaf;
                            self.internals[location as usize + indices[i]] = item | HIGH;
                            self.count += 1;
                            return None;
                        }
                        // If they are the same, we should do it again.
                    }
                    panic!(
                        "hwt::Hwt::insert(): different items not supposed to land in the same spot"
                    );
                }
            }
        }
        panic!("hwt::Hwt::insert(): got an internal node at index 6");
    }

    /// Looks up an item ID from the `Hwt`.
    ///
    /// Returns `Some(t)` if item `t` was in the `Hwt`, otherwise `None`.
    ///
    /// ```
    /// # use hwt::Hwt;
    /// let mut hwt = Hwt::new();
    /// hwt.insert(0b101, 0, |_| 0b010);
    /// hwt.insert(0b010, 1, |_| 0b101);
    /// assert_eq!(hwt.get(0b101), Some(0));
    /// assert_eq!(hwt.get(0b010), Some(1));
    /// assert_eq!(hwt.get(0b000), None);
    /// assert_eq!(hwt.get(0b111), None);
    /// ```
    pub fn get(&mut self, feature: u128) -> Option<u32> {
        // Compute the indices of the buckets and the sizes of the buckets
        // for each layer of the tree.
        let (indices, _) = indices128(feature);
        // The first index in the tree is actually the overall weight of
        // the whole number.
        let weight = feature.count_ones() as usize;
        let mut node = weight;
        for &index in &indices {
            match self.internals[node] {
                0 => return None,
                internal if internal & HIGH == 0 => node = internal as usize + index,
                leaf => return Some(leaf & !HIGH),
            }
        }
        None
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
