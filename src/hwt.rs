use crate::indices::*;
use crate::search::*;
use hashbrown::{hash_map::Entry, HashMap};
use std::cmp::{max, min};

/// This threshold determines whether to perform a brute-force search in a bucket
/// instead of a targeted search if the amount of nodes is less than this number.
///
/// Since we do a brute force search in an internal node with < `TAU` leaves,
/// this also defines the threshold at which a vector must be split into a hash table.
///
/// This should be improved by changing the threshold on a per-level of the tree basis.
const TAU: usize = 16384;

enum Internal {
    /// This always contains leaves.
    Vec(Vec<u32>),
    /// This always points to another internal node.
    Map(HashMap<usize, u32>),
}

impl Default for Internal {
    fn default() -> Self {
        Internal::Vec(Vec::with_capacity(TAU))
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
    /// `level` must be set from 0 to 6 inclusive. If it is 0, this is a bucket in the top level.
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
                let mut map = HashMap::with_capacity(TAU);
                for leaf in v.into_iter() {
                    let leaf_feature = lookup(leaf);
                    let leaf_indices = indices128(leaf_feature);
                    let new_internal = *map
                        .entry(leaf_indices[level])
                        .or_insert_with(|| self.allocate_internal());
                    if let Internal::Vec(ref mut v) =
                        self.internals[new_internal as usize]
                    {
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
        // The first index in the tree is actually the overall weight of
        // the whole number.
        let weight = feature.count_ones() as usize;
        let mut node = weight;
        let mut bucket = 0;
        let mut create_internal = false;
        #[allow(clippy::needless_range_loop)]
        for i in 0..7 {
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
                            node = indices[i];
                        }
                        Entry::Vacant(_) => {
                            create_internal = true;
                            break;
                        }
                    }
                }
            }
        }
        if create_internal {
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
                map.insert(node, new_internal);
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
        // The first index in the tree is actually the overall weight of
        // the whole number.
        let weight = feature.count_ones() as usize;
        let mut bucket = 0;
        let mut node = weight;
        for &index in &indices {
            match &self.internals[bucket] {
                Internal::Vec(vec) => {
                    return vec.iter().cloned().find(|&n| lookup(n) == feature)
                }
                Internal::Map(map) => {
                    if let Some(&occupied_node) = map.get(&node) {
                        bucket = occupied_node as usize;
                        node = index;
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
        match &self.internals[bucket] {
            Internal::Vec(v) => Box::new(
                v.iter()
                    .cloned()
                    .filter(move |&leaf| (lookup(leaf) ^ feature).count_ones() <= radius),
            ) as Box<dyn Iterator<Item = u32> + 'a>,
            Internal::Map(m) => {
                Box::new(indices.flat_map(move |(index, tws)| {
                    if let Some(&occupied_node) = m.get(&index) {
                        // The node is an internal.
                        let subbucket = occupied_node as usize;
                        either::Right(subtable(self, radius, feature, subbucket, tws, lookup))
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
