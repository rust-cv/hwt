pub struct FeatureHeap {
    cap: usize,
    size: usize,
    in_search: usize,
    search_distance: u32,
    search: u128,
    worst: u32,
    features: [Vec<u128>; 129],
}

impl FeatureHeap {
    pub fn new(cap: usize, search: u128) -> Self {
        assert_ne!(cap, 0);
        Self {
            cap,
            size: 0,
            in_search: 0,
            search_distance: 0,
            search,
            worst: 128,
            features: [
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
        }
    }

    /// Update the minimum distance we are searching at.
    pub fn search_distance(&mut self, distance: u32) {
        assert!(distance > self.search_distance);
        self.in_search += self.features[self.search_distance as usize + 1..=distance as usize]
            .iter()
            .map(Vec::len)
            .sum::<usize>();
        self.search_distance = distance;
    }

    /// Add a feature to the search.
    #[inline(always)]
    pub fn add(&mut self, feature: u128) {
        let distance = (feature ^ self.search).count_ones();
        // We stop searching once we have enough features under the search distance,
        // so if this is true it will always get added to the FeatureHeap.
        if distance <= self.search_distance {
            self.in_search += 1;
        }
        if self.size != self.cap {
            self.features[distance as usize].push(feature);
            self.size += 1;
            // Set the worst feature appropriately.
            if self.size == self.cap {
                self.update_worst();
            }
        } else if distance < self.worst {
            self.features[distance as usize].push(feature);
            self.remove_worst();
        }
    }

    #[inline(always)]
    fn update_worst(&mut self) {
        self.worst -= self.features[0..=self.worst as usize]
            .iter()
            .rev()
            .position(|v| !v.is_empty())
            .unwrap() as u32;
    }

    #[inline(always)]
    fn remove_worst(&mut self) {
        self.features[self.worst as usize].pop();
        self.update_worst();
    }

    #[inline(always)]
    pub fn in_search(&self) -> usize {
        self.in_search
    }

    pub fn fill_slice<'a>(&self, s: &'a mut [u128]) -> &'a mut [u128] {
        let total_fill = std::cmp::min(s.len(), self.size);
        for (ix, &f) in self.features.iter().flat_map(|v| v.iter()).take(total_fill).enumerate() {
            s[ix] = f;
        }
        &mut s[0..total_fill]
    }
}
