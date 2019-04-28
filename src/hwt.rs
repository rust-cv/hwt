use crate::indices::*;
use crate::search::*;
use crate::{FeatureHeap, NodeQueue};
use hashbrown::HashMap;
use log::trace;
use swar::*;

/// This threshold determines whether to perform a brute-force search in a bucket
/// instead of a targeted search if the amount of nodes is less than this number.
///
/// Since we do a brute force search in an internal node with < `TAU` leaves,
/// this also defines the threshold at which a vector must be split into a hash table.
const TAU: usize = 1 << 16;

/// The threshold at which we change to precision search for each level of the tree.
/// The reason this is different for each level is that `search_exact2` and
/// `search_exact128` have very different execution times. Higher in the tree,
/// it is cheaper to do an exact or radius search, but lower in the tree it becomes
/// incredibly expensive. Thus, this should start low and get higher so that the
/// threshold corresponds to execution complexity of the search.
const TABLE_TAUS: [usize; 7] = [0, 0, 0, 0, 0, 0, 0];

/// This determines how much space is initially allocated for a leaf vector.
const INITIAL_CAPACITY: usize = 16;

pub(crate) type InternalMap = HashMap<u128, u32, std::hash::BuildHasherDefault<ahash::AHasher>>;

#[derive(Debug)]
enum Internal {
    /// This always contains features.
    Vec(Vec<u128>),
    /// This always points to another internal node.
    Map(InternalMap),
}

impl Default for Internal {
    fn default() -> Self {
        Internal::Vec(Vec::with_capacity(INITIAL_CAPACITY))
    }
}

pub struct Hwt {
    /// A `u32` pointing to an internal node is just an index into the
    /// internals array, which is just a bump allocator for internal nodes.
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
    /// hwt.insert(0b101);
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
    /// hwt.insert(0b101);
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
    fn convert(&mut self, internal: usize, level: usize) {
        // Swap a temporary vec with the one in the store to avoid the wrath of the borrow checker.
        let mut old_vec = Internal::Vec(Vec::new());
        std::mem::swap(&mut self.internals[internal], &mut old_vec);
        // Use the old vec to create a new map for the node.
        self.internals[internal] = match old_vec {
            Internal::Vec(v) => {
                let mut map = InternalMap::default();
                for feature in v.into_iter() {
                    let index = indices128(feature)[level];
                    let new_internal =
                        *map.entry(index).or_insert_with(|| self.allocate_internal());
                    if let Internal::Vec(ref mut v) = self.internals[new_internal as usize] {
                        v.push(feature);
                    } else {
                        unreachable!(
                            "cannot have InternalStore::Map in subtable when just created"
                        );
                    }
                }
                Internal::Map(map.into_iter().collect())
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
    /// hwt.insert(0b101);
    /// hwt.insert(0b010);
    /// assert_eq!(hwt.len(), 2);
    /// ```
    pub fn insert(&mut self, feature: u128) {
        // No matter what we will insert the item, so increase the count now.
        self.count += 1;
        // Compute the indices of the buckets and the sizes of the buckets
        // for each layer of the tree.
        let indices = indices128(feature);
        let mut bucket = 0;
        let mut create_internal = None;
        for (i, &tc) in indices.iter().enumerate() {
            match &mut self.internals[bucket] {
                Internal::Vec(ref mut v) => {
                    v.push(feature);
                    if v.len() > TAU {
                        self.convert(bucket, i);
                    }
                    return;
                }
                Internal::Map(ref mut map) => {
                    match map.iter().find(|&(&tc_leaf, _)| tc == tc_leaf) {
                        Some((_, &internal)) => {
                            // Go to the next node.
                            bucket = internal as usize;
                        }
                        None => {
                            create_internal = Some(tc);
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
                v.push(feature);
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
                Internal::Vec(ref mut v) => v.push(feature),
                _ => panic!("Can't have InternalStore::Map at bottom of tree"),
            }
        }
    }

    /// Checks if a feature is in the `Hwt`.
    ///
    /// ```
    /// # use hwt::Hwt;
    /// let mut hwt = Hwt::new();
    /// hwt.insert(0b101);
    /// hwt.insert(0b010);
    /// assert!(hwt.contains(0b101));
    /// assert!(hwt.contains(0b010));
    /// assert!(!hwt.contains(0b000));
    /// assert!(!hwt.contains(0b111));
    /// ```
    pub fn contains(&mut self, feature: u128) -> bool {
        // Compute the indices of the buckets and the sizes of the buckets
        // for each layer of the tree.
        let indices = indices128(feature);
        let mut bucket = 0;
        for &index in &indices {
            match &self.internals[bucket] {
                Internal::Vec(vec) => return vec.iter().cloned().any(|n| n == feature),
                Internal::Map(map) => {
                    if let Some((_, &internal)) = map.iter().find(|&(&tc, _)| tc == index) {
                        bucket = internal as usize;
                    } else {
                        return false;
                    }
                }
            }
        }
        false
    }

    /// Find the nearest neighbors to a feature. This will give the nearest
    /// neighbors first and expand outwards. It will fill `dest` until its full
    /// with nearest neighbors in order or until `max_weight` is reached,
    /// whichever comes first. `max_error` specifies the level of weight error
    /// allowed in the output. If it is set to `0` it will always perform
    /// an exact nearest-neighbor search. Any other number will allow this
    /// method to end once it has retrieved enough features at the current
    /// search weight and up to `max_error` above the weight. These
    /// features are much more likely to be nearest neighbors, but there
    /// are no guarantees that they are, since we haven't exhausted all
    /// possible locations in the search tree.
    ///
    /// Returns the slice of filled neighbors. It may consume only
    /// part of `dest` if less neighbors are found than `dest`. It
    /// stops searching at `max_weight`, but might obtain features
    /// beyond that and still gives them to the user.
    #[allow(clippy::cognitive_complexity)]
    pub fn nearest<'a>(
        &self,
        feature: u128,
        max_weight: u32,
        max_error: u32,
        node_queue: &mut NodeQueue,
        feature_heap: &mut FeatureHeap,
        dest: &'a mut [u128],
    ) -> &'a mut [u128] {
        trace!(
            "nearest feature({:032X}) weight({})",
            feature,
            feature.count_ones()
        );
        let indices = indices128(feature);
        // Expand the root node.
        node_queue.clear();
        feature_heap.reset(dest.len(), feature);
        match &self.internals[0] {
            Internal::Vec(v) => {
                trace!("nearest sole leaf node len({})", v.len());
                for &f in v {
                    feature_heap.add(f);
                }
                return feature_heap.fill_slice(dest);
            }
            Internal::Map(m) => {
                trace!("nearest emptying root len({})", m.len());
                for (distance, node) in m
                    .iter()
                    .map(|(&tc, &node)| {
                        let distance = (tc ^ indices[0]).count_ones();
                        (distance, node)
                    })
                    .filter(|&(distance, _)| distance <= max_weight)
                {
                    match unsafe {
                        std::mem::transmute::<_, &'static Internal>(&self.internals[node as usize])
                    } {
                        Internal::Vec(v) => {
                            for &f in v {
                                feature_heap.add(f);
                            }
                            if feature_heap.done() {
                                return feature_heap.fill_slice(dest);
                            }
                        }
                        Internal::Map(m) => {
                            node_queue.add_one((distance, &m, 0));
                        }
                    }
                }
            }
        }

        for distance in 0..=max_weight {
            trace!("searching distance({})", distance);
            // Tell the feature heap we are searching at the max error distance
            // so that once we have found enough features within the error, then
            // we are done.
            feature_heap.search_distance(std::cmp::min(128, distance + max_error));
            if feature_heap.done() {
                return feature_heap.fill_slice(dest);
            }
            while node_queue.distance() == Some(distance) {
                if let Some((_, internal, level)) = node_queue.pop() {
                    if level == 7 {
                        unreachable!("hwt: it is impossible to have an internal node at layer 7");
                    }

                    trace!(
                        "nearest node distance({}) len({}) level({})",
                        distance,
                        internal.len(),
                        level
                    );
                    if internal.len() < TABLE_TAUS[level as usize] {
                        trace!("nearest brute force");
                        for (child_distance, child) in internal.iter().map(|(&tc, &child)| {
                            let child_distance = (tc ^ indices[(level + 1) as usize]).count_ones();
                            (child_distance, child)
                        }) {
                            match unsafe {
                                std::mem::transmute::<_, &'static Internal>(
                                    &self.internals[child as usize],
                                )
                            } {
                                Internal::Vec(leaves) => {
                                    for &f in leaves {
                                        feature_heap.add(f);
                                    }
                                    if feature_heap.done() {
                                        return feature_heap.fill_slice(dest);
                                    }
                                }
                                Internal::Map(m) => {
                                    node_queue.add_one((child_distance, &m, level + 1));
                                }
                            }
                        }
                    } else {
                        trace!("nearest precision search");
                        match level {
                            0 => {
                                let tp = Bits64(*internal.iter().next().unwrap().0).pack_ones();
                                for Bits64(tc) in search_exact2(
                                    64,
                                    Bits128(indices[level as usize]),
                                    Bits64(indices[level as usize + 1]),
                                    tp,
                                    distance,
                                ) {
                                    if let Some(&child) = internal.get(&tc) {
                                        match unsafe {
                                            std::mem::transmute::<_, &'static Internal>(
                                                &self.internals[child as usize],
                                            )
                                        } {
                                            Internal::Vec(leaves) => {
                                                trace!("nearest leaves len({})", leaves.len());
                                                for &f in leaves {
                                                    feature_heap.add(f);
                                                }
                                                if feature_heap.done() {
                                                    return feature_heap.fill_slice(dest);
                                                }
                                            }
                                            Internal::Map(m) => {
                                                trace!("nearest map len({})", m.len());
                                                let child_distance =
                                                    (tc ^ indices[level as usize + 1]).count_ones();
                                                node_queue.add_one((child_distance, &m, level + 1));
                                            }
                                        }
                                    }
                                }
                            }
                            1 => {
                                let tp = Bits32(*internal.iter().next().unwrap().0).pack_ones();
                                for Bits32(tc) in search_exact4(
                                    32,
                                    Bits64(indices[level as usize]),
                                    Bits32(indices[level as usize + 1]),
                                    tp,
                                    distance,
                                ) {
                                    if let Some(&child) = internal.get(&tc) {
                                        match unsafe {
                                            std::mem::transmute::<_, &'static Internal>(
                                                &self.internals[child as usize],
                                            )
                                        } {
                                            Internal::Vec(leaves) => {
                                                trace!("nearest leaves len({})", leaves.len());
                                                for &f in leaves {
                                                    feature_heap.add(f);
                                                }
                                                if feature_heap.done() {
                                                    return feature_heap.fill_slice(dest);
                                                }
                                            }
                                            Internal::Map(m) => {
                                                trace!("nearest map len({})", m.len());
                                                let child_distance =
                                                    (tc ^ indices[level as usize + 1]).count_ones();
                                                node_queue.add_one((child_distance, &m, level + 1));
                                            }
                                        }
                                    }
                                }
                            }
                            2 => {
                                let tp = Bits16(*internal.iter().next().unwrap().0).pack_ones();
                                for Bits16(tc) in search_exact8(
                                    16,
                                    Bits32(indices[level as usize]),
                                    Bits16(indices[level as usize + 1]),
                                    tp,
                                    distance,
                                ) {
                                    if let Some(&child) = internal.get(&tc) {
                                        match unsafe {
                                            std::mem::transmute::<_, &'static Internal>(
                                                &self.internals[child as usize],
                                            )
                                        } {
                                            Internal::Vec(leaves) => {
                                                trace!("nearest leaves len({})", leaves.len());
                                                for &f in leaves {
                                                    feature_heap.add(f);
                                                }
                                                if feature_heap.done() {
                                                    return feature_heap.fill_slice(dest);
                                                }
                                            }
                                            Internal::Map(m) => {
                                                trace!("nearest map len({})", m.len());
                                                let child_distance =
                                                    (tc ^ indices[level as usize + 1]).count_ones();
                                                node_queue.add_one((child_distance, &m, level + 1));
                                            }
                                        }
                                    }
                                }
                            }
                            3 => {
                                let tp = Bits8(*internal.iter().next().unwrap().0).pack_ones();
                                for Bits8(tc) in search_exact16(
                                    8,
                                    Bits16(indices[level as usize]),
                                    Bits8(indices[level as usize + 1]),
                                    tp,
                                    distance,
                                ) {
                                    if let Some(&child) = internal.get(&tc) {
                                        match unsafe {
                                            std::mem::transmute::<_, &'static Internal>(
                                                &self.internals[child as usize],
                                            )
                                        } {
                                            Internal::Vec(leaves) => {
                                                trace!("nearest leaves len({})", leaves.len());
                                                for &f in leaves {
                                                    feature_heap.add(f);
                                                }
                                                if feature_heap.done() {
                                                    return feature_heap.fill_slice(dest);
                                                }
                                            }
                                            Internal::Map(m) => {
                                                trace!("nearest map len({})", m.len());
                                                let child_distance =
                                                    (tc ^ indices[level as usize + 1]).count_ones();
                                                node_queue.add_one((child_distance, &m, level + 1));
                                            }
                                        }
                                    }
                                }
                            }
                            4 => {
                                let tp = Bits4(*internal.iter().next().unwrap().0).pack_ones();
                                for Bits4(tc) in search_exact32(
                                    4,
                                    Bits8(indices[level as usize]),
                                    Bits4(indices[level as usize + 1]),
                                    tp,
                                    distance,
                                ) {
                                    if let Some(&child) = internal.get(&tc) {
                                        match unsafe {
                                            std::mem::transmute::<_, &'static Internal>(
                                                &self.internals[child as usize],
                                            )
                                        } {
                                            Internal::Vec(leaves) => {
                                                trace!("nearest leaves len({})", leaves.len());
                                                for &f in leaves {
                                                    feature_heap.add(f);
                                                }
                                                if feature_heap.done() {
                                                    return feature_heap.fill_slice(dest);
                                                }
                                            }
                                            Internal::Map(m) => {
                                                trace!("nearest map len({})", m.len());
                                                let child_distance =
                                                    (tc ^ indices[level as usize + 1]).count_ones();
                                                node_queue.add_one((child_distance, &m, level + 1));
                                            }
                                        }
                                    }
                                }
                            }
                            5 => {
                                let tp = Bits2(*internal.iter().next().unwrap().0).pack_ones();
                                for Bits2(tc) in search_exact64(
                                    2,
                                    Bits4(indices[level as usize]),
                                    Bits2(indices[level as usize + 1]),
                                    tp,
                                    distance,
                                ) {
                                    if let Some(&child) = internal.get(&tc) {
                                        match unsafe {
                                            std::mem::transmute::<_, &'static Internal>(
                                                &self.internals[child as usize],
                                            )
                                        } {
                                            Internal::Vec(leaves) => {
                                                trace!("nearest leaves len({})", leaves.len());
                                                for &f in leaves {
                                                    feature_heap.add(f);
                                                }
                                                if feature_heap.done() {
                                                    return feature_heap.fill_slice(dest);
                                                }
                                            }
                                            Internal::Map(m) => {
                                                trace!("nearest map len({})", m.len());
                                                let child_distance =
                                                    (tc ^ indices[level as usize + 1]).count_ones();
                                                node_queue.add_one((child_distance, &m, level + 1));
                                            }
                                        }
                                    }
                                }
                            }
                            6 => {
                                let tp = Bits1(*internal.iter().next().unwrap().0).pack_ones();
                                for Bits1(tc) in search_exact128(
                                    1,
                                    Bits2(indices[level as usize]),
                                    Bits1(indices[level as usize + 1]),
                                    tp,
                                    distance,
                                ) {
                                    if let Some(&child) = internal.get(&tc) {
                                        match unsafe {
                                            std::mem::transmute::<_, &'static Internal>(
                                                &self.internals[child as usize],
                                            )
                                        } {
                                            Internal::Vec(leaves) => {
                                                trace!("nearest leaves len({})", leaves.len());
                                                for &f in leaves {
                                                    feature_heap.add(f);
                                                }
                                                if feature_heap.done() {
                                                    return feature_heap.fill_slice(dest);
                                                }
                                            }
                                            Internal::Map(m) => {
                                                trace!("nearest map len({})", m.len());
                                                let child_distance =
                                                    (tc ^ indices[level as usize + 1]).count_ones();
                                                node_queue.add_one((child_distance, &m, level + 1));
                                            }
                                        }
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                        // Add the internal back at the next possible distance.
                        if distance != 128 {
                            node_queue.add_one((distance + 1, internal, level));
                        }
                    }
                }
            }
        }
        feature_heap.fill_slice(dest)
    }

    /// Find all neighbors within a given radius.
    pub fn search_radius<'a>(
        &'a self,
        radius: u32,
        feature: u128,
    ) -> impl Iterator<Item = u128> + 'a {
        let index = indices128(feature)[1];
        // Iterate over every applicable index in the root.
        self.bucket_scan_radius(radius, feature, 0, Self::radius2, move |tc| {
            (tc ^ index).count_ones() <= radius
        })
    }

    fn radius2<'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
    ) -> impl Iterator<Item = u128> + 'a {
        let index = indices128(feature)[2];
        self.bucket_scan_radius(radius, feature, bucket, Self::radius4, move |tc| {
            (tc ^ index).count_ones() <= radius
        })
    }

    fn radius4<'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
    ) -> impl Iterator<Item = u128> + 'a {
        let index = indices128(feature)[3];
        self.bucket_scan_radius(radius, feature, bucket, Self::radius8, move |tc| {
            (tc ^ index).count_ones() <= radius
        })
    }

    fn radius8<'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
    ) -> impl Iterator<Item = u128> + 'a {
        let index = indices128(feature)[4];
        self.bucket_scan_radius(radius, feature, bucket, Self::radius16, move |tc| {
            (tc ^ index).count_ones() <= radius
        })
    }

    fn radius16<'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
    ) -> impl Iterator<Item = u128> + 'a {
        let index = indices128(feature)[5];
        self.bucket_scan_radius(radius, feature, bucket, Self::radius32, move |tc| {
            (tc ^ index).count_ones() <= radius
        })
    }

    fn radius32<'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
    ) -> impl Iterator<Item = u128> + 'a {
        let index = indices128(feature)[6];
        self.bucket_scan_radius(radius, feature, bucket, Self::radius64, move |tc| {
            (tc ^ index).count_ones() <= radius
        })
    }

    fn radius64<'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
    ) -> impl Iterator<Item = u128> + 'a {
        let index = indices128(feature)[7];
        self.bucket_scan_radius(radius, feature, bucket, Self::radius128, move |tc| {
            (tc ^ index).count_ones() <= radius
        })
    }

    fn radius128<'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
    ) -> impl Iterator<Item = u128> + 'a {
        self.bucket_scan_radius(
            radius,
            feature,
            bucket,
            |_, _, _, bucket| -> Box<dyn Iterator<Item = u128> + 'a> {
                panic!(
                    "hwt::Hwt::neighbors128(): it is an error to find an internal node this far down in the tree (bucket: {})", bucket, 
                )
            },
            move |tc| panic!("hwt::Hwt::neighbors128(): it is an error to find an internal node this far down in the tree (tc: {})", tc)
        )
    }

    /// Search the given `bucket` with the `indices` iterator, using `subtable`
    /// to recursively iterate over buckets found inside this bucket.
    #[allow(clippy::too_many_arguments)]
    fn bucket_scan_radius<'a, I: 'a>(
        &'a self,
        radius: u32,
        feature: u128,
        bucket: usize,
        subtable: fn(&'a Self, u32, u128, usize) -> I,
        filter: impl Fn(u128) -> bool + 'a,
    ) -> Box<dyn Iterator<Item = u128> + 'a>
    where
        I: Iterator<Item = u128>,
    {
        trace!(
            "bucket_scan_radius feature({:032X}) radius({}) bucket({})",
            feature,
            radius,
            bucket,
        );
        let lookup_distance = move |leaf: u128| (leaf ^ feature).count_ones();
        match &self.internals[bucket] {
            Internal::Vec(v) => Box::new(
                v.iter()
                    .cloned()
                    .filter(move |&leaf| lookup_distance(leaf) <= radius),
            ),
            Internal::Map(m) => Box::new(
                m.iter()
                    .filter(move |&(&key, _)| filter(key))
                    .flat_map(move |(_, &node)| subtable(self, radius, feature, node as usize)),
            ),
        }
    }
}

impl Default for Hwt {
    fn default() -> Self {
        Self {
            internals: vec![Internal::default()],
            count: 0,
        }
    }
}
