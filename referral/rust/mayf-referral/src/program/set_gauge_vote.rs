use mayf_fixed_number::FixedNumber;

use crate::{Amount, Emission, GaugeVote, Lane, Protocol, Result};

pub struct SetGaugeVoteArgs<N> {
    pub protocol: Protocol<N>,
    pub lane: Lane<N>,
    pub vote: GaugeVote,
    pub votes: Amount,
    pub emission: Emission<N>,
}

pub struct SetGaugeVoteReceipt<N> {
    pub protocol_next: Protocol<N>,
    pub lane_next: Lane<N>,
    pub vote_next: GaugeVote,
}

/// Settle old weights first. The host authenticates the voter's exclusively held
/// voting power and keys the vote to this lane. Gauge policy is separate from
/// referral accounting; this action implements proportional allocation.
pub fn set_gauge_vote<N: FixedNumber>(args: SetGaugeVoteArgs<N>) -> Result<SetGaugeVoteReceipt<N>> {
    args.protocol.check_lane(args.lane.id)?;
    if args.emission.at < args.lane.settled_at {
        return Err(crate::Error::TimeWentBack);
    }
    let (gauge, lane_gauge, allocation) =
        args.protocol
            .gauge
            .revote(args.lane.gauge, args.vote.votes, args.votes, args.emission)?;
    let protocol_next = Protocol {
        gauge,
        ..args.protocol
    };
    let lane_next = Lane {
        gauge: lane_gauge,
        pool: args.lane.pool.absorb(allocation)?,
        settled_at: args.emission.at,
        ..args.lane
    };
    Ok(SetGaugeVoteReceipt {
        protocol_next,
        lane_next,
        vote_next: GaugeVote { votes: args.votes },
    })
}
