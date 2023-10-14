use fixedbitset::FixedBitSet;
use strum::IntoEnumIterator;

use crate::{BackgroundFlag, BackgroundFlagIter};

pub struct FixedBitSetIterator<'a> {
    bitset: &'a FixedBitSet,
    iter: BackgroundFlagIter,
}

impl<'a> FixedBitSetIterator<'a> {
    pub fn new(bitset: &'a FixedBitSet) -> Self {
        Self {
            bitset,
            iter: BackgroundFlag::iter(),
        }
    }
}

impl Iterator for FixedBitSetIterator<'_> {
    type Item = BackgroundFlag;

    fn next(&mut self) -> Option<Self::Item> {
        for flag in &mut self.iter {
            if self.bitset.contains(flag as usize) {
                return Some(flag);
            }
        }
        None
    }
}
