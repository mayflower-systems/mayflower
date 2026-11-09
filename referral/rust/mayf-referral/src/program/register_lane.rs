use mayf_fixed_number::FixedNumber;

use crate::{Error, Lane, LaneGauge, LaneId, LanePool, MAX_LANES, Protocol, Result};

pub struct RegisterLaneArgs<N> {
    pub protocol: Protocol<N>,
}

pub struct RegisterLaneReceipt<N> {
    pub protocol_next: Protocol<N>,
    pub new_lane: Lane<N>,
}

/// Register one permanent asset slot. A lane earns nothing until it has weight.
pub fn register_lane<N: FixedNumber>(args: RegisterLaneArgs<N>) -> Result<RegisterLaneReceipt<N>> {
    let mut protocol = args.protocol;
    if protocol.lane_count >= MAX_LANES {
        return Err(Error::TooManyLanes);
    }
    let id = LaneId::new(protocol.lane_count)?;
    protocol.lane_count += 1;
    let new_lane = Lane {
        id,
        gauge: LaneGauge {
            weight_index_seen: protocol.gauge.weight_index,
            ..LaneGauge::default()
        },
        pool: LanePool::default(),
        settled_at: protocol.gauge.emission_seen.at,
    };
    Ok(RegisterLaneReceipt {
        protocol_next: protocol,
        new_lane,
    })
}
