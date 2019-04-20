//! This is a special priority queue specifically for 128-bit hamming weight searches.
//! 
//! This queue works by having 129 vectors, one for each distance. When we find that an internal node
//! achieves a distance of `n` at the least, we place the index of that node into the vector associated
//! with that distance. Any time we take a node off, we place all of its children into the appropriate
//! distance priorities.
//! 
//! We maintain the lowest weight vector at any given time in the queue. When a vector runs out,
//! because of the nature of hamming weight trees, we are guaranteed that nothing will ever have a distance
//! that low again, since the sum of the distance of bit substrings can only be higher than the distance of
//! their parents. This means we only have to move the lowest weight vector forwards. Also, typically every
//! removal will be constant time since we are incredibly likely to find all the nearest neighbor's required
//! before we reach a distance of 64, which is the lowest possible max distance in the root node (distances
//! of the hamming weights 0-64 and 64-128). The more things in the hamming weight tree, the less likely
//! this becomes. Assuming randomly distributed features, we expect half of the features to have a distance
//! below 64, so it is incredibly likely that all removals are constant time since we will always encounter
//! a removal below or equal to 64.

use std::fmt;

#[derive(Clone)]
pub struct NodeQueue {
    distances: [Vec<(u128, u32, u8)>; 129],
    lowest: usize,
}

impl NodeQueue {
    /// Takes all the entries in the root node (level 0) and adds them to the queue.
    /// 
    /// This is passed the (distance, node) pairs.
    pub fn new(root: impl IntoIterator<Item = (u32, u128, u32)>) -> Self {
        let mut new = Self {
            distances: [vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![],vec![]],
            lowest: 0,
        };
        new.add(root.into_iter().map(|(distance, tp, node)| (distance, tp, node, 0)));
        new
    }

    pub fn pop(&mut self) -> Option<(u32, u128, u32, u8)> {
        loop {
            if let Some((tp, node, level)) = self.distances[self.lowest].pop() {
                return Some((self.lowest as u32, tp, node, level));
            } else if self.lowest == 128 {
                return None;
            } else {
                self.lowest += 1;
            }
        }
    }

    /// Takes an iterator over (distance, tp, node, level)
    pub fn add(&mut self, children: impl IntoIterator<Item = (u32, u128, u32, u8)>) {
        for child in children {
            self.add_one(child);
        }
    }

    /// Takes an iterator over (distance, tp, node, level)
    pub fn add_one(&mut self, (distance, tp, node, level): (u32, u128, u32, u8)) {
        self.distances[distance as usize].push((tp, node, level));
    }
}

impl fmt::Debug for NodeQueue {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.distances[..].fmt(formatter)
    }
}
