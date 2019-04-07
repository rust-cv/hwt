use crate::indices::*;
use crate::search::*;
use arrayvec::ArrayVec;
use hashbrown::{
    hash_map::{self, Entry},
    HashMap,
};
use std::cmp::{max, min};

const HIGH: u32 = 0x8000_0000;
/// This threshold determines whether to perform a brute-force search in a bucket
/// instead of a targeted search if the amount of nodes is less than this number.
///
/// This should be improved by changing the threshold on a per-level of the tree basis.
const FULL_SEARCH_THRESHOLD: usize = 1024;

#[derive(Default)]
struct Internal {
    map: HashMap<usize, u32>,
    count: usize,
}

pub struct Hwt {
    /// If a `u32` has a high bit set to `1` then it is a leaf node, otherwise it is an internal node.
    /// A `u32` pointing to an internal node is just an index into the internals array, which is
    /// just a bump allocator for internal nodes. It is possible to have more than 2^31 entries, but
    /// 2^31 internal nodes cannot be exceeded.
    internals: Vec<Internal>,
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
        self.internals[0].count
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
        self.len() == 0
    }

    /// Decreases the count of every node descending down to a particular index.
    /// This should be used to undo the counting up if there is a duplicate insert
    /// or when removing things from the Hwt.
    fn remove_count(&mut self, _indices: [usize; 7]) {
        unimplemented!()
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
        let (indices, _, _) = indices128(feature);
        // The first index in the tree is actually the overall weight of
        // the whole number.
        let weight = feature.count_ones() as usize;
        let mut node = weight;
        let mut bucket = 0;
        for i in 0..7 {
            match self.internals[bucket].map.entry(node) {
                Entry::Occupied(o) => {
                    let occupied_node = *o.get();
                    // If its an internal node.
                    if occupied_node & HIGH == 0 {
                        let internal = occupied_node;
                        // Increase the bucket by 1 before we descend since we will be inserting a node.
                        self.internals[bucket].count += 1;
                        // Go to the next node.
                        bucket = internal as usize;
                        node = indices[i];
                    } else {
                        // It is a leaf node.
                        let leaf = occupied_node;
                        // Increase the internals by `1` before we go further.
                        self.internals[bucket].count += 1;
                        // Get the leaf's indices. The size of any table we care
                        // about is the same as this `item`.
                        let leaf_feature = lookup(leaf & !HIGH);
                        let (leaf_indices, _, _) = indices128(leaf_feature);
                        // Iterate and make more child nodes until the `item` and
                        // the leaf differ.
                        for i in i..7 {
                            // Allocate the space for the next bucket.
                            // This will always be the same between both items.
                            let location = self.internals.len() as u32;
                            // Ensure the bucket index hasn't gotten larger than
                            // the max.
                            assert!(location & HIGH == 0);
                            // Create the new bucket.
                            let mut new_bucket = Internal::default();
                            // Set the new bucket to 2 since we are putting 2 nodes into it.
                            new_bucket.count = 2;
                            // Add the bucket to this parent node.
                            // We already accounted for the count change by assigning 2 to new buckets
                            // and incrementing the first bucket's count by 1.
                            self.internals[bucket].map.insert(node, location);
                            // Update the node to be this node.
                            bucket = location as usize;
                            node = indices[i];
                            // Check if the indices are different.
                            if leaf_indices[i] != indices[i] {
                                // If they finally differ, we can add them to
                                // different spots and return.
                                new_bucket.map.insert(leaf_indices[i], leaf);
                                new_bucket.map.insert(indices[i], item | HIGH);
                                // Push the new bucket before returning.
                                self.internals.push(new_bucket);
                                return None;
                            }
                            // If they are the same, we should do it again.
                            // Push the new bucket.
                            self.internals.push(new_bucket);
                        }
                        panic!(
                            "hwt::Hwt::insert(): different items not supposed to land in the same spot: {:X}, {:X}, {:X?}, {:X?}",
                            feature, leaf_feature, indices, leaf_indices
                        );
                    }
                }
                Entry::Vacant(v) => {
                    v.insert(item | HIGH);
                    // Increase the internals by `1`.
                    self.internals[bucket].count += 1;
                    return None;
                }
            }
        }
        match self.internals[bucket].map.entry(node) {
            // If it is occupied, then it can only be a leaf. We replace that leaf.
            Entry::Occupied(o) => Some(o.replace_entry(item | HIGH).1),
            // A vacant entry should be replaced.
            Entry::Vacant(v) => {
                v.insert(item | HIGH);
                dbg!("TODO: If a node is replaced, the tree count is now invalid.");
                // Increase the internals by `1`.
                self.internals[bucket].count += 1;
                // TODO: Call remove_count() here after it works!
                None
            }
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
        let (indices, _, _) = indices128(feature);
        // The first index in the tree is actually the overall weight of
        // the whole number.
        let weight = feature.count_ones() as usize;
        let mut bucket = 0;
        let mut node = weight;
        for &index in &indices {
            if let Some(&occupied_node) = self.internals[bucket].map.get(&node) {
                if occupied_node & HIGH == 0 {
                    // It is internal.
                    bucket = occupied_node as usize;
                    node = index;
                } else {
                    // It is a leaf.
                    return Some(occupied_node & !HIGH);
                }
            } else {
                return None;
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
        if self.internals[bucket].count < FULL_SEARCH_THRESHOLD {
            // If we have very few leaves, we want to do a brute-force search.
            Box::new(
                self.bucket_brute_force(bucket)
                    .filter(move |&leaf| (lookup(leaf) ^ feature).count_ones() <= radius),
            ) as Box<dyn Iterator<Item = u32> + 'a>
        } else {
            Box::new(indices.flat_map(move |(index, tws)| {
                if let Some(&occupied_node) = self.internals[bucket].map.get(&index) {
                    if occupied_node & HIGH != 0 {
                        // The node is a leaf.
                        let leaf = occupied_node & !HIGH;
                        either::Left(if (lookup(leaf) ^ feature).count_ones() <= radius {
                            Some(leaf).into_iter()
                        } else {
                            None.into_iter()
                        })
                    } else {
                        // The node is an internal.
                        let subbucket = occupied_node as usize;
                        either::Right(subtable(self, radius, feature, subbucket, tws, lookup))
                    }
                } else {
                    either::Left(None.into_iter())
                }
            })) as Box<dyn Iterator<Item = u32> + 'a>
        }
    }

    fn bucket_brute_force<'a>(&'a self, bucket: usize) -> impl Iterator<Item = u32> + 'a {
        // Make the Vec with the maximum capacity we will need.
        let mut bucket_stack: ArrayVec<[hash_map::Iter<'_, usize, u32>; 8]> = ArrayVec::new();
        bucket_stack.push(self.internals[bucket].map.iter());
        std::iter::from_fn(move || {
            while let Some(mut iter) = bucket_stack.pop() {
                if let Some((_, &sub)) = iter.next() {
                    bucket_stack.push(iter);
                    if sub & HIGH == 0 {
                        // It is an internal node.
                        bucket_stack.push(self.internals[sub as usize].map.iter())
                    } else {
                        // It is a leaf.
                        return Some(sub & !HIGH);
                    }
                }
            }
            None
        })
    }
}

impl Default for Hwt {
    fn default() -> Self {
        // The number of child nodes of the root is determined by the different
        // possible hamming weights. The maximum hamming weight is the number
        // of bits and the minimum is 0, so this means that there are
        // `NBits + 1` child nodes.
        Self {
            internals: vec![Internal::default()],
        }
    }
}
