use core::num::NonZeroU128;

use mayf_fixed_number::{DirectedRounding::Floor, FixedNumber};

use crate::{Amount, Error, Lane, LaneId, LanePool, MAX_LANES, Protocol, Result};

pub struct RegisterLaneArgs<N> {
    pub protocol: Protocol,
    /// Starting cumulative earnings. Earlier earnings never belong to this lane.
    pub total_earned: N,
}

pub struct RegisterLaneReceipt<N> {
    pub protocol_next: Protocol,
    pub new_lane: Lane<N>,
}

/// Register a permanent asset slot at its current earnings baseline.
/// Choose a wide accounting driver: cumulative earnings may exceed `u64::MAX`.
pub fn register_lane<N: FixedNumber>(args: RegisterLaneArgs<N>) -> Result<RegisterLaneReceipt<N>> {
    let mut protocol = args.protocol;
    if protocol.lane_count >= MAX_LANES {
        return Err(Error::TooManyLanes);
    }
    if args.total_earned < N::ZERO {
        return Err(Error::InvalidEarnings);
    }
    let grid = NonZeroU128::new(1 << 64).ok_or(Error::NumberOutOfRange)?;
    if N::checked_from_ratio_round(1, grid, Floor)? == N::ZERO {
        return Err(Error::GridTooCoarse);
    }
    // Whole balances and rational-rate intermediates must fit. Lifetime bounds
    // remain a host choice; all subsequent arithmetic is checked.
    N::checked_from_u64(Amount::MAX)?.checked_mul_floor(N::checked_from_u64(1_000_000_000_000)?)?;
    let id = LaneId::new(protocol.lane_count)?;
    protocol.lane_count += 1;
    let new_lane = Lane {
        id,
        pool: LanePool::default(),
        total_earned_seen: args.total_earned,
    };
    Ok(RegisterLaneReceipt {
        protocol_next: protocol,
        new_lane,
    })
}

#[cfg(test)]
mod tests;
