use crate::indices::*;
use crate::search::*;
use hashbrown::{hash_map::Entry, HashMap};
use std::cmp::{max, min};
use swar::*;

/// This threshold determines whether to perform a brute-force search in a bucket
/// instead of a targeted search if the amount of nodes is less than this number.
///
/// Since we do a brute force search in an internal node with < `TAU` leaves,
/// this also defines the threshold at which a vector must be split into a hash table.
///
/// This should be improved by changing the threshold on a per-level of the tree basis.
const TAU: usize = 16384;

/// This determines how much space is initially allocated for a leaf vector.
const INITIAL_CAPACITY: usize = 16;

enum Internal {
    /// This always contains leaves.
    Vec(Vec<u32>),
    /// This always points to another internal node.
    Map(HashMap<u128, u32, std::hash::BuildHasherDefault<ahash::AHasher>>),
}

impl Default for Internal {
    fn default() -> Self {
        Internal::Vec(Vec::with_capacity(INITIAL_CAPACITY))
    }
}

pub struct Hwt {
    /// If a `u32` has a high bit set to `1` then it is a leaf node, otherwise it is an internal node.
    /// A `u32` pointing to an internal node is just an index into the internals array, which is
    /// just a bump allocator for internal nodes. It is possible to have more than 2^31 entries, but
    /// 2^31 internal nodes cannot be exceeded.
    internals: Vec<Internal>,
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
        self.len() == 0
    }

    fn allocate_internal(&mut self) -> u32 {
        let internal = self.internals.len() as u32;
        assert!(internal < std::u32::MAX);
        self.internals.push(Internal::default());
        internal
    }

    /// Converts an internal node from a `Vec` of leaves to a `HashMap` from indices to internal nodes.
    ///
    /// `internal` must be the internal node index which should be replaced
    /// `level` must be set from 0 to 7 inclusive. If it is 0, this is the root.
    /// `lookup` must allow looking up the feature of leaves.
    fn convert<F>(&mut self, internal: usize, level: usize, mut lookup: F)
    where
        F: FnMut(u32) -> u128,
    {
        // Swap a temporary vec with the one in the store to avoid the wrath of the borrow checker.
        let mut old_vec = Internal::Vec(Vec::new());
        std::mem::swap(&mut self.internals[internal], &mut old_vec);
        // Use the old vec to create a new map for the node.
        self.internals[internal] = match old_vec {
            Internal::Vec(v) => {
                let mut map = HashMap::default();
                for leaf in v.into_iter() {
                    let leaf_feature = lookup(leaf);
                    let index = indices128(leaf_feature)[level];
                    let new_internal =
                        *map.entry(index).or_insert_with(|| self.allocate_internal());
                    if let Internal::Vec(ref mut v) = self.internals[new_internal as usize] {
                        v.push(leaf);
                    } else {
                        unreachable!(
                            "cannot have InternalStore::Map in subtable when just created"
                        );
                    }
                }
                Internal::Map(map)
            }
            _ => panic!("tried to convert an InternalStore::Map"),
        }
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
    pub fn insert<F>(&mut self, feature: u128, item: u32, mut lookup: F)
    where
        F: FnMut(u32) -> u128,
    {
        // No matter what we will insert the item, so increase the count now.
        self.count += 1;
        // Compute the indices of the buckets and the sizes of the buckets
        // for each layer of the tree.
        let indices = indices128(feature);
        let mut bucket = 0;
        let mut create_internal = None;
        for (i, &node) in indices.iter().enumerate() {
            match &mut self.internals[bucket] {
                Internal::Vec(ref mut v) => {
                    v.push(item);
                    if v.len() > TAU {
                        self.convert(bucket, i, &mut lookup);
                    }
                    return;
                }
                Internal::Map(ref mut map) => {
                    match map.entry(node) {
                        Entry::Occupied(o) => {
                            let internal = *o.get();
                            // Go to the next node.
                            bucket = internal as usize;
                        }
                        Entry::Vacant(_) => {
                            create_internal = Some(node);
                            break;
                        }
                    }
                }
            }
        }
        if let Some(vacant_node) = create_internal {
            // Allocate a new internal Vec node.
            let new_internal = self.allocate_internal();
            // Add the item to the new internal Vec.
            if let Internal::Vec(ref mut v) = self.internals[new_internal as usize] {
                v.push(item);
            } else {
                unreachable!("cannot have InternalStore::Map in subtable when just created");
            }
            // Add the new internal to the vacant map spot.
            if let Internal::Map(ref mut map) = &mut self.internals[bucket] {
                map.insert(vacant_node, new_internal);
            } else {
                unreachable!("shouldn't ever get vec after finding vacant map node");
            }
        } else {
            // We are just adding this item to the bottom of the tree in a Vec.
            match self.internals[bucket] {
                Internal::Vec(ref mut v) => v.push(item),
                _ => panic!("Can't have InternalStore::Map at bottom of tree"),
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
    /// let lookup = |n| match n { 0 => 0b101, 1 => 0b010, _ => panic!() };
    /// hwt.insert(0b101, 0, lookup);
    /// hwt.insert(0b010, 1, lookup);
    /// assert_eq!(hwt.get(0b101, lookup), Some(0));
    /// assert_eq!(hwt.get(0b010, lookup), Some(1));
    /// assert_eq!(hwt.get(0b000, lookup), None);
    /// assert_eq!(hwt.get(0b111, lookup), None);
    /// ```
    pub fn get<F>(&mut self, feature: u128, mut lookup: F) -> Option<u32>
    where
        F: FnMut(u32) -> u128,
    {
        // Compute the indices of the buckets and the sizes of the buckets
        // for each layer of the tree.
        let indices = indices128(feature);
        let mut bucket = 0;
        for index in &indices {
            match &self.internals[bucket] {
                Internal::Vec(vec) => return vec.iter().cloned().find(|&n| lookup(n) == feature),
                Internal::Map(map) => {
                    if let Some(&occupied_node) = map.get(index) {
                        bucket = occupied_node as usize;
                    } else {
                        return None;
                    }
                }
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
        let start = max(0, sw - radius as i32) as u128;
        let end = min(128, sw + radius as i32) as u128;
        // Iterate over every applicable index in the root.
        self.bucket_scan(
            radius,
            feature,
            0,
            lookup,
            // The index is the `tw` because at the root node indices
            // are target weights.
            start..=end,
            Self::radius2,
        )
    }

    fn radius2<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tp: u128,
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        let indices = indices128(feature);
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search_radius2(Bits128(indices[0]), Bits64(indices[1]), Bits128(tp), radius)
                .map(|(tc, _sod)| tc.0),
            Self::radius4,
        )
    }

    fn radius4<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tp: u128,
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        let indices = indices128(feature);
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search_radius4(Bits64(indices[1]), Bits32(indices[2]), Bits64(tp), radius)
                .map(|(tc, _sod)| tc.0),
            Self::radius8,
        )
    }

    fn radius8<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tp: u128,
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        let indices = indices128(feature);
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search_radius8(Bits32(indices[2]), Bits16(indices[3]), Bits32(tp), radius)
                .map(|(tc, _sod)| tc.0),
            Self::radius16,
        )
    }

    fn radius16<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tp: u128,
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        let indices = indices128(feature);
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search_radius16(Bits16(indices[3]), Bits8(indices[2]), Bits16(tp), radius)
                .map(|(tc, _sod)| tc.0),
            Self::radius32,
        )
    }

    fn radius32<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tp: u128,
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        let indices = indices128(feature);
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search_radius32(Bits8(indices[4]), Bits4(indices[3]), Bits8(tp), radius)
                .map(|(tc, _sod)| tc.0),
            Self::radius64,
        )
    }

    fn radius64<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tp: u128,
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        let indices = indices128(feature);
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search_radius64(Bits4(indices[5]), Bits2(indices[4]), Bits4(tp), radius)
                .map(|(tc, _sod)| tc.0),
            Self::radius128,
        )
    }

    fn radius128<'a, F: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        tp: u128,
        lookup: &'a F,
    ) -> impl Iterator<Item = u32> + 'a
    where
        F: Fn(u32) -> u128,
    {
        let indices = indices128(feature);
        self.bucket_scan(
            radius,
            feature,
            bucket,
            lookup,
            search_radius128(Bits2(indices[6]), Bits1(indices[5]), Bits2(tp), radius).map(|(tc, _sod)| tc.0),
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
    fn bucket_scan<'a, F: 'a, I: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        lookup: &'a F,
        indices: impl Iterator<Item = u128> + 'a,
        subtable: impl Fn(&'a Self, u32, u128, usize, u128, &'a F) -> I + 'a,
    ) -> Box<dyn Iterator<Item = u32> + 'a>
    where
        F: Fn(u32) -> u128,
        I: Iterator<Item = u32>,
    {
        match &self.internals[bucket] {
            Internal::Vec(v) => Box::new(
                v.iter()
                    .cloned()
                    .filter(move |&leaf| (lookup(leaf) ^ feature).count_ones() <= radius),
            ) as Box<dyn Iterator<Item = u32> + 'a>,
            Internal::Map(m) => {
                Box::new(indices.flat_map(move |tc| {
                    if let Some(&occupied_node) = m.get(&tc) {
                        // The node is an internal.
                        let subbucket = occupied_node as usize;
                        either::Right(subtable(self, radius, feature, subbucket, tc, lookup))
                    } else {
                        either::Left(None.into_iter())
                    }
                })) as Box<dyn Iterator<Item = u32> + 'a>
            }
        }
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
            count: 0,
        }
    }
}
