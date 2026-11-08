use mayf_fixed_number::FixedNumber;

use crate::{Amount, GaugeVote, Lane, Protocol, Result, Seconds};

pub struct SetGaugeVoteArgs<N> {
    pub protocol: Protocol<N>,
    pub lane: Lane<N>,
    /// The voter's vote on the lane, or the default if the host has none
    pub vote: GaugeVote,
    /// The voter's new votes on the lane
    pub votes: Amount,
    pub now: Seconds,
}

pub struct SetGaugeVoteReceipt<N> {
    pub protocol_next: Protocol<N>,
    pub lane_next: Lane<N>,
    pub vote_next: GaugeVote,
}

/// Set one voter's votes on one lane, from `now` on. The lane earns at its old weight up to
/// `now`. The host bounds each voter's votes, for example by its voting power.
pub fn set_gauge_vote<N>(args: SetGaugeVoteArgs<N>) -> Result<SetGaugeVoteReceipt<N>>
where
    N: FixedNumber,
{
    let (protocol, lane) = (args.protocol, args.lane);
    let emitted = protocol.emitter.emitted_at(args.now);
    let (old, new) = (args.vote.votes, args.votes);
    let (gauge, lane_gauge, allocation) = protocol.gauge.revote(lane.gauge, old, new, emitted)?;

    let protocol_next = Protocol { gauge, ..protocol };
    let lane_next = Lane {
        gauge: lane_gauge,
        pool: lane.pool.absorb(allocation)?,
    };

    Ok(SetGaugeVoteReceipt {
        protocol_next,
        lane_next,
        vote_next: GaugeVote { votes: new },
    })
}
