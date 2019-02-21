use bitarray::typenum::Unsigned;
use bitarray::BitArray;

use std::marker::PhantomData;

pub struct Hwt<NBits> {
    /// The u32 points to a location in the internals array that is the
    /// start of a slice of internal or leaf node indices.
    internals: Vec<u32>,
    count: usize,
    /// Because the structure of `internals` depends on the number of bits, we
    /// must force `Hwt` to be paramaterized by the number of bits.
    phantom: PhantomData<NBits>,
}

impl<NBits> Hwt<NBits>
where
    NBits: Unsigned,
{
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
}

impl<NBits> Default for Hwt<NBits>
where
    NBits: Unsigned,
{
    fn default() -> Self {
        // The number of child nodes of the root is determined by the different
        // possible hamming weights. The maximum hamming weight is the number
        // of bits and the minimum is 0, so this means that there are
        // `NBits + 1` child nodes.
        Self {
            internals: vec![0; NBits::to_usize() + 1],
            count: 0,
            phantom: PhantomData,
        }
    }
}
