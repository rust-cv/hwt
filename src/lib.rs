//! # `Hwt`
//!
//! The Hamming Weight Tree was originally implemented in the paper
//! "Online Nearest Neighbor Search in Hamming Space" by
//! Sepehr Eghbali, Hassan Ashtiani, and Ladan Tahvildari. This is an attempt
//! to improve on the performance and encapsulate the implementation in a Rust
//! crate for easy consumption.
//!
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
//!        5                   [1-4]                   2
//!    3       2         [1-2]       [0-2]         1       1
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
//!
//! # Searching
//!
//! To limit the search space, we depend on the fact that the sum of the
//! absolute differences of hamming weights of substrings cannot exceed
//! the sum of the hamming distances of substrings. This means if the
//! sum of the absolute differences in hamming weights between the
//! bucket index's implicit weights at any given level of the tree
//! exceeds `radius` then we know it is impossible for any results to be
//! found in that branch of the tree. This allows us to filter what we
//! search to be only nodes that could theoretically match.
//!
//! For the top level, its clear to scan (`weight-radius..=weight+radius`).
//! This is because results cannot be found outside where the weight differs
//! by more than `radius`. For the levels below that it becomes more
//! complicated to search the bucket. To do so, let us consider the case of
//! `L = 0` (the 0th level starts after looking up the bucket for the overall
//! hamming weight).
//!
//! Lets say we have a 128-bit feature with this tree of hamming weights:
//! ```no_build
//!   5
//! 3   2
//! ```
//!
//! If we want to search for things in `radius <= 1` then at the top level we
//! search `4..6`. Let us consider what happens when we then try to search the
//! bucket found at index `4`. At this point we have a situation where the left
//! side could vary in `0..=4`, since we have a 128-bit number, each half can
//! easily fit `4` ones. However, we dont need to search all of these
//! possibilites.
//!
//! If the left side were to have a weight of `1` then the right
//! side would have a weight of `3`. Remember "the sum of the absolute
//! differences of hamming weights of substrings cannot exceed the sum of
//! the hamming distances of substrings." If we look at our search point, we
//! find that the sum of the differences is `abs(3 - 1) + abs(2 - 3) = 3`.
//! This is greater than our search radius of `1`, therefore it is impossible
//! to find a number with a hamming distance within the radius there.
//!
//! Now consider what happens if we go to a weight of `2` on the left side.
//! In this case we have `2` bits on the right side. The sum of the differences
//! is `abs(3 - 2) + abs(2 - 2) = 1`. This is equal to our search radius and
//! therefore it is possible to find a match in that bucket.
//!
//! In conclusion, we need to iterate in `2..=3`. This has limited the
//! possibilities greatly. However, we need to know how to derive this range.
//!
//! What we are going to find specifically is the way to derive the range of
//! the left substring weight (not the actual bucket index) that allows just
//! that substring to fit inside of a `radius`. We will use this primitive to
//! derive the solution for any number of substrings.
//!
//! Let the weight of the target parent substring be `TW`.
//!
//! Let the weight of the target left substring be `TL`.
//!
//! Let the weight of the target right substring be `TR`.
//!
//! Let the weight of the search parent substring be `SW`.
//!
//! Let the weight of the search left substring be `SL`.
//!
//! Let the weight of the search right substring be `SR`.
//!
//! `TR = TW - TL`
//!
//! `SR = SW - SL`
//!
//! Let the sum of substring weight differences be `SOD`.
//!
//! `SOD = abs(TL - SL) + abs(TR - SR)`
//!
//! We are searching for `TL` that satisfy `SOD <= radius`. The `SOD` has two
//! inflection points that come from the two `abs` in its expression. Between
//! those two inflection points there are only four possible combinations:
//!
//! 1. `TL` is going towards `SL` and `TR` is going towards `SR` (slope `-2`).
//! 2. `TL` hits its its inflection point first and starts going away from `SL`
//!     and `TR` is still going towards `SR` (slope `0`).
//! 3. `TR` hits its its inflection point first and starts going away from `SR`
//!     and `TL` is still going towards `SL` (slope `0`).
//! 4. `TL` and `TR` have both hit their inflection points and are going away
//!     from `SL` and `SR` respectively (slope `2`).
//!
//! As we can see, regardless of whether `TL` or `TR` hit their inflection
//! point first, we can be guaranteed that the slope is `0` before the final
//! inflection point. This happens because `TL` and `TR` are inversely related.
//!
//! We must start by computing where the first slope would intersect with the
//! radius. We assume that `TL` is below or equal to `SL` and that `TR` is
//! above or equal to `SR`. Given this, we know that when
//! `(SL - TL) + (TR - SR) = radius` we enter the search area. Since
//! `TR = TW - TL` we can rewrite this as
//! `(SL - TL) + (TW - TL - SR) = radius`. Since `SR`, `SL`, and `TW` are
//! all known at this point, we can solve for `TL`:
//!
//! `TL = (-radius + SL - SR + TW) / 2`
//!
//! Lets do the same thing for the opposite case (slope `2` reaches `radius`):
//!
//! `(TL - SL) + (SR - TR) = radius`
//!
//! `(TL - SL) + (SR - TW + TL) = radius`
//!
//! `TL = (radius + SL - SR + TW) / 2`
//!
//! We can see that there is a shared intercept between the two equations, but
//! we will not extract the intercept directly because we wouldnt get the same
//! result if we divide by 2 before adding since we would loose a bit of
//! precision.
//!
//! Let `C = SL - SR + TW`.
//!
//! We must search in `(-radius + C) / 2..=(radius + C) / 2`. However, this
//! makes the assumption that there are any matches. It is possible that the
//! radius is low enough that we get no matches. In this case we can test the
//! `0` slope case. We just need to test if `TL = (radius + C) / 2` is
//! actually a match. To test that:
//!
//! `abs((radius + C) / 2 - SL) + abs(TW - (radius + C) / 2 - SR) <= radius`.
//!
//! If the test succeeds, then we can safely iterate over the range.
//!
//! Lets apply this reasoning to the previously mentioned tree. We expect to
//! get the range `2..=3`.
//!
//! `C = SL - SR + TW = 3 - 2 + 4 = 5`
//!
//! Now we need to test
//! `abs((radius + C) / 2 - SL) + abs(TW - (radius + C) / 2 - SR) <= radius`.
//!
//! `abs((1 + 5) / 2 - 3) + abs(4 - (1 + 5) / 2 - 2) <= 1`
//!
//! `abs(6 / 2 - 3) + abs(4 - 6 / 2 - 2) <= 1`
//!
//! `abs(0) + abs(-1) <= 1`
//!
//! `1 <= 1`
//!
//! The test passes. Now we compute the range.
//!
//! `(-radius + C) / 2..=(radius + C) / 2`
//!
//! `(-1 + 5) / 2..=(1 + 5) / 2`
//!
//! `4 / 2..=6 / 2`
//!
//! `2..=3`
//!
//! This is the range we expected.
//!
//! We may need to clip the range to be inside the bucket as well, since the
//! radius might cover a bigger set of hamming distances than the range.
//!
//! Now we wish to find all combinations of substrings that result in getting
//! below the radius. To do this we need to know the `SOD` at each index we
//! search in a given substring. To do that we must describe the relationship
//! between `TL` and `SOD`.
//!
//! There are three phases in the iteration pattern over `TL`. The first is
//! when the `radius` is going down, the second is when it stays flat, the
//! third is when it is going up. The test in the last part made sure the
//! bottom was above the radius. We need to compute the points at which the
//! slope becomes 0, which are the inflection points. Luckily, these are
//! trivial to calculate. They are when the inside of the `abs` expressions
//! in `SOD` is equal to `0`:
//!
//! `TL - SL = 0`
//!
//! `TR - SR = 0`
//!
//! We also know that `TR = TW - TL`, so we can rewrite this in terms of `TL`:
//!
//! `TW - TL - SR = 0`
//!
//! We care about `TL` when we hit the inflection point:
//!
//! `TL = SL`
//!
//! `TL = -SR + TW`
//!
//! We dont care which inflection point we hit first, we just want to know
//! where it is. We can just take the `min` and `max` of these two
//! expressions to get the beginning and ending of the flat part of the curve.
//!
//! Now we want to solve for the `SOD`. Just like last time, we start with `TL`
//! being lower that `SL` and `TR` being higher than `SR`.
//!
//! `(SL - TL) + (TW - TL - SR) = SOD`
//!
//! `(TL - SL) + (SR - TW + TL) = SOD`
//!
//! We can simplify these to make it a bit clearer:
//!
//! `C = SL - SR + TW`
//!
//! `-2TL + C = SOD`
//!
//! `2TL - C = SOD`
//!
//! It starts by going down with a slope of `-2` and ends going up with a slope
//! of `2` just like we expect.
//!
//! We can use this expression to compute the `SOD` for each part of iteration.
//!
//! Now the iteration is split into three parts:
//!
//! `(-radius + C) / 2..SL` (`SOD = -2TL + C`)
//! `SL..-SR + TW` (`SOD = -2SL + C`)
//! `-SR + TW..=(radius + C) / 2` (`SOD = 2TL - C`)
//!
//! At this point we can compute the `SOD` over all of our input indices. Now
//! we iterate over all input indices specificed, compute their `SOD`, and then
//! perform a search over subsequent substrings by passing them a `new_radius`
//! of `new_radius = radius - SOD`. This guarantees that all paths in that
//! substring also dont exceed the total `SOD` for all substrings in the level.

pub mod indices;
mod search;

use indices::*;
pub use search::*;
use std::cmp::{max, min};
use std::iter::repeat;

const HIGH: u32 = 0x8000_0000;

pub struct Hwt {
    /// The u32 points to a location in the `internal_indices` array that is
    /// the start of a slice of internal or leaf node indices. If an internal
    /// has a high bit set to `1` then it is a leaf node.
    internals: Vec<u32>,
    /// Contains the full width index into the `internals`.
    internal_indices: Vec<usize>,
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
    /// - `F`: A function which should give the `feature` for the given ID.
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
                    node = self.internal_indices[internal as usize] + indices[i];
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
                    let leaf_feature = lookup(leaf & !HIGH);
                    let (leaf_indices, _) = indices128(leaf_feature);
                    // Iterate and make more child nodes until the `item` and
                    // the leaf differ.
                    for i in i..7 {
                        // Allocate the space for the next bucket.
                        // This will always be the same between both items.
                        let internal_location = self.internals.len();
                        // Ensure the bucket index hasn't gotten larger than
                        // the max.
                        let location = self.internal_indices.len() as u32;
                        assert!(location & HIGH == 0);
                        self.internals.extend(repeat(0).take(sizes[i]));
                        self.internal_indices.push(internal_location);
                        // Add the bucket to this parent node.
                        self.internals[node] = location;
                        // Update the node to be this node.
                        node = internal_location + indices[i];
                        // Check if the indices are different.
                        if leaf_indices[i] != indices[i] {
                            // If they finally differ, we can add them to
                            // different spots and return.
                            self.internals[internal_location + leaf_indices[i]] = leaf;
                            self.internals[node] = item | HIGH;
                            self.count += 1;
                            return None;
                        }
                        // If they are the same, we should do it again.
                    }
                    panic!(
                        "hwt::Hwt::insert(): different items not supposed to land in the same spot: {:X}, {:X}, {:X?}, {:X?}",
                        feature, leaf_feature, indices, leaf_indices
                    );
                }
            }
        }
        match self.internals[node] {
            0 => {
                self.internals[node] = item | HIGH;
                self.count += 1;
                None
            }
            leaf if leaf & HIGH != 0 => {
                self.internals[node] = item | HIGH;
                Some(leaf & !HIGH)
            }
            _ => panic!("hwt::Hwt::insert(): got an internal node at end of tree"),
        }
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
                internal if internal & HIGH == 0 => {
                    node = self.internal_indices[internal as usize] + index;
                }
                leaf => return Some(leaf & !HIGH),
            }
        }
        None
    }

    /// Find the nearest neighbors to a feature. This will give the nearest
    /// neighbors first and expand outwards. This evaluates lazily, so use
    /// `Iterator::take()` to just take as many as you need.
    pub fn nearest<'a, F: 'a>(
        &'a self,
        feature: u128,
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        (0..=128)
            .map(move |r| {
                self.search_radius(r, feature, lookup)
                    .filter(move |&n| (lookup(n) ^ feature).count_ones() == r)
            })
            .flatten()
    }

    /// Find all neighbors within a given radius.
    pub fn search_radius<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        // Manually compute the range of `tw` (which is also index)
        // for the root node since it is unique.
        let sw = feature.count_ones() as i32;
        let start = max(0, sw - radius as i32) as u32;
        let end = min(128, sw + radius as i32) as u32;
        // Iterate over every applicable index in the root.
        self.bucket_scan(
            radius,
            feature,
            0,
            lookup,
            // The index is the `tw` because at the root node indices
            // are target weights.
            (start..=end).map(|tw| (tw as usize, [tw])),
            Self::neighbors2,
        )
    }

    /// Find all neighbors in a bucket at depth `0` of the tree
    /// (`-1` is the root) with a hamming distance less or equal to `radius`.
    fn neighbors2<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tws: [u32; 1],
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        // The number of bits per substring.
        const NBITS: u32 = 128 / 2;
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search2(NBITS, feature, tws[0], radius).map(|(index, _, _, tws)| (index, tws)),
            Self::neighbors4,
        )
    }

    /// Find all neighbors in a bucket at depth `1` of the tree
    /// (`-1` is the root) with a hamming distance less or equal to `radius`.
    fn neighbors4<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tws: [u32; 2],
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        // The number of bits per substring.
        const NBITS: u32 = 128 / 4;
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search4(NBITS, feature, tws, radius).map(|(index, _, _, tws)| (index, tws)),
            Self::neighbors8,
        )
    }

    /// Find all neighbors in a bucket at depth `2` of the tree
    /// (`-1` is the root) with a hamming distance less or equal to `radius`.
    fn neighbors8<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tws: [u32; 4],
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        // The number of bits per substring.
        const NBITS: u32 = 128 / 8;
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search8(NBITS, feature, tws, radius).map(|(index, _, _, tws)| (index, tws)),
            Self::neighbors16,
        )
    }

    /// Find all neighbors in a bucket at depth `3` of the tree
    /// (`-1` is the root) with a hamming distance less or equal to `radius`.
    fn neighbors16<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tws: [u32; 8],
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        // The number of bits per substring.
        const NBITS: u32 = 128 / 16;
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search16(NBITS, feature, tws, radius).map(|(index, _, _, tws)| (index, tws)),
            Self::neighbors32,
        )
    }

    /// Find all neighbors in a bucket at depth `4` of the tree
    /// (`-1` is the root) with a hamming distance less or equal to `radius`.
    fn neighbors32<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tws: [u32; 16],
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        // The number of bits per substring.
        const NBITS: u32 = 128 / 32;
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search32(NBITS, feature, tws, radius).map(|(index, _, _, tws)| (index, tws)),
            Self::neighbors64,
        )
    }

    /// Find all neighbors in a bucket at depth `5` of the tree
    /// (`-1` is the root) with a hamming distance less or equal to `radius`.
    fn neighbors64<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tws: [u32; 32],
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        // The number of bits per substring.
        const NBITS: u32 = 128 / 64;
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search64(NBITS, feature, tws, radius).map(|(index, _, _, tws)| (index, tws)),
            Self::neighbors128,
        )
    }

    /// Find all neighbors in a bucket at depth `6` of the tree
    /// (`-1` is the root) with a hamming distance less or equal to `radius`.
    fn neighbors128<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tws: [u32; 64],
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search128(feature, tws, radius).map(|index| (index, ())),
            // We just outright lie about the type there because otherwise
            // it can't infer the type.
            |_, _, _, bucket, _, _| -> Box<dyn Iterator<Item = u32> + 'a> {
                panic!(
                    "hwt::Hwt::neighbors128(): it is an error to find an internal node this far down in the tree (bucket: {})", bucket, 
                )
            },
        )
    }

    /// Search the given `bucket` with the `indices` iterator, using `subtable`
    /// to recursively iterate over buckets found inside this bucket.
    #[allow(clippy::too_many_arguments)]
    fn bucket_scan<'a, F: 'a, I: 'a, TWS: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        lookup: &'a F,
        indices: impl Iterator<Item = (usize, TWS)> + 'a,
        subtable: impl Fn(&'a Self, u32, u128, usize, TWS, &'a F) -> I + 'a,
    ) -> Box<dyn Iterator<Item = u32> + 'a>
    where
        F: Fn(u32) -> u128,
        I: Iterator<Item = u32>,
        TWS: Clone,
    {
        Box::new(indices.flat_map(move |(index, tws)| {
            match self.internals[bucket + index] {
                // Empty
                0 => either::Left(None.into_iter()),
                // Leaf
                leaf if leaf & HIGH != 0 => either::Left({
                    let leaf = leaf & !HIGH;
                    if (lookup(leaf) ^ feature).count_ones() <= radius {
                        Some(leaf).into_iter()
                    } else {
                        None.into_iter()
                    }
                }),
                // Internal
                internal => either::Right({
                    let subbucket = self.internal_indices[internal as usize];
                    subtable(self, radius, feature, subbucket, tws, lookup)
                }),
            }
        })) as Box<dyn Iterator<Item = u32> + 'a>
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
            internal_indices: vec![0],
            count: 0,
        }
    }
}
