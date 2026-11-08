//! One hour of emissions, using the wiki's examples.
//!
//! 100 credits are emitted per hour, and Mayflower-ETH has 10% of the gauge votes, so its lane
//! receives 10 credits. Alice took a top-level seat, Bob joined under Alice, and Charlie joined
//! under Bob. Each holds 100 Mayflower-ETH. With a 10% override rate, they earn 4, 3.33, and
//! 2.67 credits. Alice shares 50% of what she earns with Bob, and Bob shares 25% of what he earns
//! with Charlie. Bob keeps Alice's gift whole, so they keep 2, 4.5, and 3.5 credits.
//!
//! The example plays the host. It keeps every record in memory, and each of its methods runs
//! one action.

use std::collections::BTreeMap;
use std::iter;
use std::num::NonZero;

use mayf_fixed_drivers::U192F64;
use mayf_referral::program::{
    CollectCreditsArgs, CreateProtocolArgs, DepositIntoLaneArgs, JoinAtTopLevelArgs,
    JoinUnderParentArgs, RegisterLaneArgs, SetGaugeVoteArgs, SetSharingArgs, SettleNodeInLaneArgs,
    collect_credits, create_protocol, deposit_into_lane, join_at_top_level, join_under_parent,
    register_lane, set_gauge_vote, set_sharing, settle_node_in_lane,
};
use mayf_referral::{
    Amount, EmissionRate, GaugeVote, Lane, LaneNode, MicroBasisPoints, NetworkTerms, Node, Offer,
    Protocol, Result, Seconds,
};

/// This example names nodes and voters with strings.
type Id = &'static str;

/// This example numbers lanes in the order they register.
type LaneId = usize;

/// This example's fixed-point type. It holds every value the kernel computes.
type Number = U192F64;

/// Credits and Mayflower-ETH both have 9 decimals.
const UNIT: Amount = 1_000_000_000;
const HOUR: Seconds = 3_600;

/// Every seat here is free: top-level seats, and seats under each node.
const FREE: Offer = Offer {
    current_fee: 0,
    increment: 0,
};

fn main() -> Result<()> {
    let percent = |value| MicroBasisPoints::from_percent(value).expect("a percentage up to 100");
    let rate = EmissionRate {
        amount: 100 * UNIT,
        period: NonZero::new(HOUR).expect("an hour is not zero"),
    };
    let mut host = Host::new(NetworkTerms::new(2, percent(10))?, rate)?;

    // Mayflower-ETH has 100 of the 1,000 gauge votes. No one holds the other lane, so its
    // 90 credits go unallocated.
    let eth = host.register_lane();
    let other = host.register_lane();
    host.set_gauge_vote("voter", eth, 100)?;
    host.set_gauge_vote("voter", other, 900)?;

    host.join_at_top_level("alice");
    host.join_under_parent("alice", "bob")?;
    host.join_under_parent("bob", "charlie")?;

    // A member's sharing applies in every lane. Each lane picks it up when it settles, and each
    // deposit settles the lane.
    host.set_sharing("alice", percent(50));
    host.set_sharing("bob", percent(25));
    for member in ["alice", "bob", "charlie"] {
        host.deposit_into_lane(eth, member, 100 * UNIT)?;
    }

    // One hour later, each member settles the lane and collects.
    host.now = HOUR;
    for member in ["alice", "bob", "charlie"] {
        host.settle_node_in_lane(eth, member)?;
    }
    let collected = [
        host.collect_credits("alice")?,
        host.collect_credits("bob")?,
        host.collect_credits("charlie")?,
    ];

    // Rounding never pays more than was emitted, so each collects one base unit less than
    // the ideal. That fraction of a base unit stays in the member's node for its next
    // collection.
    assert_eq!(
        collected,
        [2 * UNIT - 1, 9 * UNIT / 2 - 1, 7 * UNIT / 2 - 1]
    );
    println!("Alice, Bob, and Charlie collect {collected:?} credit base units.");
    Ok(())
}

/// What a lane action needs about a member: its node, its lineage, and the lane nodes the host
/// has for them in the lane, by node id
type MemberInLane = (
    Node<Id, Number>,
    Vec<Node<Id, Number>>,
    Vec<(Id, LaneNode<Number>)>,
);

/// A host that keeps every record in memory. Each method loads the records one action needs,
/// runs the action, and stores every record in its receipt.
struct Host {
    /// The clock, which the example moves by hand
    now: Seconds,
    protocol: Protocol<Number>,
    lanes: Vec<Lane<Number>>,
    gauge_votes: BTreeMap<(Id, LaneId), GaugeVote>,
    nodes: BTreeMap<Id, Node<Id, Number>>,
    lane_nodes: BTreeMap<(LaneId, Id), LaneNode<Number>>,
}

impl Host {
    fn new(terms: NetworkTerms, emission_rate: EmissionRate) -> Result<Self> {
        let created = create_protocol(CreateProtocolArgs {
            terms,
            emission_rate,
            top_level_offer: FREE,
            now: 0,
        })?;

        Ok(Self {
            now: 0,
            protocol: created.new_protocol,
            lanes: Vec::new(),
            gauge_votes: BTreeMap::new(),
            nodes: BTreeMap::new(),
            lane_nodes: BTreeMap::new(),
        })
    }

    fn register_lane(&mut self) -> LaneId {
        let registered = register_lane(RegisterLaneArgs);
        self.lanes.push(registered.new_lane);
        self.lanes.len() - 1
    }

    fn set_gauge_vote(&mut self, voter: Id, lane: LaneId, votes: Amount) -> Result<()> {
        let vote = self.gauge_votes.get(&(voter, lane)).cloned();
        let voted = set_gauge_vote(SetGaugeVoteArgs {
            protocol: self.protocol.clone(),
            lane: self.lanes[lane].clone(),
            vote: vote.unwrap_or_default(),
            votes,
            now: self.now,
        })?;

        self.protocol = voted.protocol_next;
        self.lanes[lane] = voted.lane_next;
        self.gauge_votes.insert((voter, lane), voted.vote_next);
        Ok(())
    }

    fn join_at_top_level(&mut self, member: Id) {
        let joined = join_at_top_level(JoinAtTopLevelArgs {
            protocol: self.protocol.clone(),
            new_node_id: member,
            new_node_offer: FREE,
        });

        self.protocol = joined.protocol_next;
        self.nodes.insert(member, joined.new_node);
    }

    fn join_under_parent(&mut self, parent: Id, member: Id) -> Result<()> {
        let joined = join_under_parent(JoinUnderParentArgs {
            protocol: self.protocol.clone(),
            parent: self.nodes[parent].clone(),
            lineage: self.lineage(parent),
            new_node_id: member,
            new_node_offer: FREE,
        })?;

        self.nodes.insert(parent, joined.parent_next);
        self.nodes.insert(member, joined.new_node);
        Ok(())
    }

    fn deposit_into_lane(&mut self, lane: LaneId, member: Id, amount: Amount) -> Result<()> {
        let (member, lineage, lane_nodes) = self.member_in_lane(lane, member);
        let deposited = deposit_into_lane(DepositIntoLaneArgs {
            protocol: self.protocol.clone(),
            lane: self.lanes[lane].clone(),
            member,
            lineage,
            lane_nodes,
            amount,
            now: self.now,
        })?;

        self.lanes[lane] = deposited.lane_next;
        self.store_nodes(deposited.member_next, deposited.lineage_next);
        self.store_lane_nodes(lane, deposited.lane_nodes_next);
        Ok(())
    }

    fn settle_node_in_lane(&mut self, lane: LaneId, node: Id) -> Result<()> {
        let (node, lineage, lane_nodes) = self.member_in_lane(lane, node);
        let settled = settle_node_in_lane(SettleNodeInLaneArgs {
            protocol: self.protocol.clone(),
            lane: self.lanes[lane].clone(),
            node,
            lineage,
            lane_nodes,
            now: self.now,
        })?;

        self.store_nodes(settled.node_next, settled.lineage_next);
        self.store_lane_nodes(lane, settled.lane_nodes_next);
        Ok(())
    }

    fn set_sharing(&mut self, member: Id, sharing: MicroBasisPoints) {
        let shared = set_sharing(SetSharingArgs {
            member: self.nodes[member].clone(),
            sharing,
        });

        self.nodes.insert(member, shared.member_next);
    }

    /// Returns the credits to mint to the member's owner.
    fn collect_credits(&mut self, member: Id) -> Result<Amount> {
        let collected = collect_credits(CollectCreditsArgs {
            member: self.nodes[member].clone(),
        })?;

        self.nodes.insert(member, collected.member_next);
        Ok(collected.credits_collected)
    }

    /// A node's lineage: its parent, its parent's parent, and so on up to a top-level member
    fn lineage(&self, id: Id) -> Vec<Node<Id, Number>> {
        let mut lineage = Vec::new();
        let mut parent_id = self.nodes[id].parent_id;

        while let Some(id) = parent_id {
            let parent = self.nodes[id].clone();
            parent_id = parent.parent_id;
            lineage.push(parent);
        }

        lineage
    }

    fn member_in_lane(&self, lane: LaneId, member: Id) -> MemberInLane {
        let member = self.nodes[member].clone();
        let lineage = self.lineage(member.id);
        let lane_nodes = iter::once(&member)
            .chain(&lineage)
            .filter_map(|node| {
                let lane_node = self.lane_nodes.get(&(lane, node.id))?;
                Some((node.id, lane_node.clone()))
            })
            .collect();

        (member, lineage, lane_nodes)
    }

    fn store_nodes(&mut self, member: Node<Id, Number>, lineage: Vec<Node<Id, Number>>) {
        for node in iter::once(member).chain(lineage) {
            self.nodes.insert(node.id, node);
        }
    }

    fn store_lane_nodes(&mut self, lane: LaneId, lane_nodes: Vec<(Id, LaneNode<Number>)>) {
        for (id, lane_node) in lane_nodes {
            self.lane_nodes.insert((lane, id), lane_node);
        }
    }
}
