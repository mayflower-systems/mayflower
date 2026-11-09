use crate::{Error, Result};

pub const MAX_LANES: u8 = 16;

/// Permanent slot for an asset. Registration assigns consecutive IDs.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LaneId(u8);

impl LaneId {
    pub fn new(slot: u8) -> Result<Self> {
        if slot >= MAX_LANES {
            return Err(Error::InvalidLane);
        }
        Ok(Self(slot))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Every lane ever materialized for a node, including lanes now holding zero.
/// Keeping these bits makes a complete family settlement verifiable in O(16).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LaneMask(u16);

impl LaneMask {
    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub const fn contains(self, lane: LaneId) -> bool {
        self.0 & (1 << lane.get()) != 0
    }

    pub(crate) fn insert(&mut self, lane: LaneId) {
        self.0 |= 1 << lane.get();
    }
}
