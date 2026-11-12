//! A two-asset walkthrough of the target referral rules.
//!
//! Alice holds 100 BTC units, Bob holds 100 BTC units, and Carol holds 100 ETH
//! units. The host supplies 200 BTC-lane and 100 ETH-lane credits, giving
//! each member's assets 100 gross credits. With q=10%, Alice shares half of her
//! gross overrides and Bob shares a quarter: net rewards are 110, 107.5, 82.5.
//!
//! This in-memory host stores action receipts. A real host must additionally
//! authenticate all inputs and atomically apply each receipt's custody effects.
//! Credit escrow and auction payments use already-issued credits; collecting
//! the net rewards below is the only action that authorizes minting new credits.

use std::collections::BTreeMap;
use std::iter;

use mayf_fixed_drivers::U192F64;
use mayf_fixed_number::FixedNumber;
use mayf_referral::program::{
    CollectCreditsArgs, CreateProtocolArgs, JoinAtTopLevelArgs, JoinUnderParentArgs,
    RegisterLaneArgs, SetBalanceArgs, SetCreditStakeArgs, SetSharingArgs, SettleNodeInLaneArgs,
    collect_credits, create_protocol, join_at_top_level, join_under_parent, register_lane,
    set_balance, set_credit_stake, set_sharing, settle_node_in_lane,
};
use mayf_referral::{
    Amount, Lane, LaneId, LaneNode, MicroBasisPoints, NetworkTerms, Node, Offer, Protocol, Result,
    RootSeat,
};

type Id = &'static str;
type Number = U192F64;

/// Ten base units per displayed credit. Asset quantities have their own units.
const UNIT: Amount = 10;
const FREE: Offer = Offer {
    current_fee: 0,
    increment: 0,
};

fn main() -> Result<()> {
    let percent = |value| MicroBasisPoints::from_percent(value).expect("valid percentage");
    let mut host = Host::new(NetworkTerms::new(2, percent(10))?);
    let btc = host.register_lane()?;
    let eth = host.register_lane()?;

    // The host supplies the winning auction award, stores its consumed state,
    // and burns the returned price. Ordinary child joins can remain free.
    host.activate_award(RootSeat::Awarded {
        node_id: "alice",
        price: UNIT,
    })?;
    host.join("alice", "bob")?;
    host.join("bob", "carol")?;

    host.sharing("alice", percent(50))?;
    host.sharing("bob", percent(25))?;
    // These principals are 100 and 200 credit BASE units (10 and 20 displayed
    // credits). This makes the example's binary gift indexes exact. Principal
    // is escrowed, never burned, and incoming gifts do not compound into it.
    host.stake("alice", "bob", 100)?;
    host.stake("bob", "carol", 200)?;
    host.balance(btc, "alice", 100)?;
    host.balance(btc, "bob", 100)?;
    host.balance(eth, "carol", 100)?;

    // These authenticated totals are data. Referral need not know how the host
    // obtained them; no schedule, votes, or clock enters the library's actions.
    host.earnings
        .insert(btc, Number::checked_from_u64(200 * UNIT)?);
    host.earnings
        .insert(eth, Number::checked_from_u64(100 * UNIT)?);
    host.settle(btc, "bob")?;
    host.settle(eth, "carol")?;
    let collected = [
        host.collect("alice")?,
        host.collect("bob")?,
        host.collect("carol")?,
    ];
    assert_eq!(collected, [110 * UNIT, 1_075, 825]);
    println!(
        "Alice, Bob, Carol collect {collected:?} credit base units (110, 107.5, 82.5 credits)."
    );
    Ok(())
}

type MemberInLane = (
    Node<Id, Number>,
    Vec<Node<Id, Number>>,
    Vec<(Id, LaneNode<Number>)>,
);
type FamilySources = (Vec<(Lane<Number>, Number)>, Vec<(LaneId, LaneNode<Number>)>);

struct Host {
    protocol: Protocol,
    lanes: BTreeMap<LaneId, Lane<Number>>,
    earnings: BTreeMap<LaneId, Number>,
    seats: BTreeMap<Id, RootSeat<Id>>,
    nodes: BTreeMap<Id, Node<Id, Number>>,
    positions: BTreeMap<(LaneId, Id), LaneNode<Number>>,
}

impl Host {
    fn new(terms: NetworkTerms) -> Self {
        let receipt = create_protocol(CreateProtocolArgs { terms });
        Self {
            protocol: receipt.new_protocol,
            lanes: BTreeMap::new(),
            earnings: BTreeMap::new(),
            seats: BTreeMap::new(),
            nodes: BTreeMap::new(),
            positions: BTreeMap::new(),
        }
    }

    fn register_lane(&mut self) -> Result<LaneId> {
        let receipt = register_lane(RegisterLaneArgs {
            protocol: self.protocol,
            total_earned: Number::ZERO,
        })?;
        let id = receipt.new_lane.id;
        self.protocol = receipt.protocol_next;
        self.lanes.insert(id, receipt.new_lane);
        self.earnings.insert(id, Number::ZERO);
        Ok(id)
    }

    fn activate_award(&mut self, seat: RootSeat<Id>) -> Result<()> {
        let receipt = join_at_top_level(JoinAtTopLevelArgs {
            seat,
            new_node_offer: FREE,
        })?;
        let id = receipt.new_node.id;
        // A production host burns this amount atomically with these writes.
        println!(
            "Auction award for {id}: burn {} issued credit base units.",
            receipt.credits_to_burn
        );
        self.seats.insert(id, receipt.seat_next);
        self.nodes.insert(id, receipt.new_node);
        Ok(())
    }

    fn join(&mut self, parent: Id, child: Id) -> Result<()> {
        let receipt = join_under_parent(JoinUnderParentArgs {
            protocol: self.protocol,
            parent: self.nodes[parent].clone(),
            new_node_id: child,
            new_node_offer: FREE,
            maximum_fee: 0,
        })?;
        // `join_fee` transfers from the newcomer to its parent; it is not burned.
        println!(
            "{child} joins {parent} for {} credit base units.",
            receipt.join_fee
        );
        self.nodes.insert(parent, receipt.parent_next);
        self.nodes.insert(child, receipt.new_node);
        Ok(())
    }

    fn sharing(&mut self, id: Id, sharing: MicroBasisPoints) -> Result<()> {
        let (lanes, lane_nodes) = self.sources(id);
        let receipt = set_sharing(SetSharingArgs {
            protocol: self.protocol,
            member: self.nodes[id].clone(),
            lanes,
            lane_nodes,
            sharing,
        })?;
        self.store_lanes(receipt.lanes_next);
        self.nodes.insert(id, receipt.member_next);
        self.store_sources(id, receipt.lane_nodes_next);
        Ok(())
    }

    fn stake(&mut self, parent: Id, child: Id, stake: Amount) -> Result<()> {
        let (lanes, parent_lane_nodes) = self.sources(parent);
        let receipt = set_credit_stake(SetCreditStakeArgs {
            protocol: self.protocol,
            parent: self.nodes[parent].clone(),
            child: self.nodes[child].clone(),
            lanes,
            parent_lane_nodes,
            stake,
        })?;
        // Apply `escrow_change` to issued-credit custody; no minting is allowed.
        self.store_lanes(receipt.lanes_next);
        self.nodes.insert(parent, receipt.parent_next);
        self.nodes.insert(child, receipt.child_next);
        self.store_sources(parent, receipt.parent_lane_nodes_next);
        Ok(())
    }

    fn balance(&mut self, lane: LaneId, id: Id, balance: Amount) -> Result<()> {
        let (member, lineage, lane_nodes) = self.member_in_lane(lane, id);
        let receipt = set_balance(SetBalanceArgs {
            protocol: self.protocol,
            lane: self.lanes[&lane].clone(),
            member,
            lineage,
            lane_nodes,
            balance,
            total_earned: self.earnings[&lane],
        })?;
        // Reconcile `balance_change` with the authoritative internal position.
        self.lanes.insert(lane, receipt.lane_next);
        self.store_nodes(receipt.member_next, receipt.lineage_next);
        self.store_positions(lane, receipt.lane_nodes_next);
        Ok(())
    }

    fn settle(&mut self, lane: LaneId, id: Id) -> Result<()> {
        let (node, lineage, lane_nodes) = self.member_in_lane(lane, id);
        let receipt = settle_node_in_lane(SettleNodeInLaneArgs {
            protocol: self.protocol,
            lane: self.lanes[&lane].clone(),
            node,
            lineage,
            lane_nodes,
            total_earned: self.earnings[&lane],
        })?;
        self.store_nodes(receipt.node_next, receipt.lineage_next);
        self.store_positions(lane, receipt.lane_nodes_next);
        Ok(())
    }

    fn collect(&mut self, id: Id) -> Result<Amount> {
        let receipt = collect_credits(CollectCreditsArgs {
            member: self.nodes[id].clone(),
        })?;
        self.nodes.insert(id, receipt.member_next);
        // Mint only this already-accrued net amount, never the gross allocation.
        Ok(receipt.credits_collected)
    }

    fn member_in_lane(&self, lane: LaneId, id: Id) -> MemberInLane {
        let member = self.nodes[id].clone();
        let mut lineage = Vec::new();
        let mut parent = member.parent_id;
        while let Some(id) = parent {
            let node = self.nodes[id].clone();
            parent = node.parent_id;
            lineage.push(node);
        }
        let positions = iter::once(&member)
            .chain(&lineage)
            .filter_map(|node| {
                self.positions
                    .get(&(lane, node.id))
                    .cloned()
                    .map(|state| (node.id, state))
            })
            .collect();
        (member, lineage, positions)
    }

    /// Exactly the permanent mask, including any position with zero balances.
    fn sources(&self, id: Id) -> FamilySources {
        let lanes: Vec<_> = self
            .lanes
            .values()
            .filter(|lane| self.nodes[id].lanes.contains(lane.id))
            .map(|lane| (lane.clone(), self.earnings[&lane.id]))
            .collect();
        let positions = lanes
            .iter()
            .map(|(lane, _)| (lane.id, self.positions[&(lane.id, id)].clone()))
            .collect();
        (lanes, positions)
    }

    fn store_nodes(&mut self, member: Node<Id, Number>, lineage: Vec<Node<Id, Number>>) {
        for node in iter::once(member).chain(lineage) {
            self.nodes.insert(node.id, node);
        }
    }

    fn store_positions(&mut self, lane: LaneId, positions: Vec<(Id, LaneNode<Number>)>) {
        for (id, position) in positions {
            self.positions.insert((lane, id), position);
        }
    }

    fn store_sources(&mut self, id: Id, positions: Vec<(LaneId, LaneNode<Number>)>) {
        for (lane, position) in positions {
            self.positions.insert((lane, id), position);
        }
    }

    fn store_lanes(&mut self, lanes: Vec<Lane<Number>>) {
        for lane in lanes {
            self.lanes.insert(lane.id, lane);
        }
    }
}
