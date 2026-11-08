#![no_std]
#![forbid(unsafe_code)]

//! The Mayflower referral and emissions kernel, as a pure state machine.
//!
//! Credits are emitted at a known rate. Gauge votes split them among lanes, where a lane is
//! one deposit token, such as Mayflower-ETH. Members form a referral tree with no root node:
//! top-level members are siblings, and every other member joined directly under a parent. Every
//! lane runs along all of the tree's links, like the fibers of a bundle. In each lane, the tree
//! divides the lane's credits among the members who hold it, by their balances there. Each
//! member pays an override to its upline, and shares a portion of what it earns with its direct
//! children. There is one credits token, so a member's sharing and the credits it keeps belong
//! to its node, across all lanes. The wiki's Credit Emissions and Referral Network pages specify
//! the rules.
//!
//! # Layers
//!
//! The kernel has three layers. Each keeps an index, a running total per unit, and the layer
//! below uses that index as its clock:
//! - Emission turns seconds into the total credits emitted.
//! - Weighting turns credits emitted into credits per gauge vote, and so into each lane's
//!   allocation.
//! - The tree turns a lane's allocation into credits per unit of balance, and so into each
//!   member's rewards and gifts.
//!
//! A record stores the index it last saw, and earns its weight times the index's growth since
//! then. So an action settles only the records it touches. A vote touches one lane. A balance
//! change touches one lane, and the nodes and lane nodes of the member and its lineage, which
//! the network's maximum depth bounds. No action loads a sibling, a child, or another lane.
//!
//! # Records
//!
//! The host stores five kinds of record:
//! - one [`Protocol`]: the network's terms, the price of a top-level seat, the emitter, and the
//!   gauge
//! - one [`Lane`] per lane
//! - one [`GaugeVote`] per voter and lane
//! - one [`Node`] per member, by node id: its parent's id, its offer, its sharing, and the
//!   credits it keeps from every lane
//! - one [`LaneNode`] per lane and node: the node's balances in that lane, stored once an action
//!   first returns it
//!
//! A [`Protocol`] holds an [`Emitter`] and a [`Gauge`], and a [`Lane`] holds a [`LaneGauge`] and
//! a [`LanePool`]. Records are plain data with public fields. The kernel does not serialize
//! them: the host stores their fields however it likes, and rebuilds each record from exactly
//! the fields it stored. Only the actions in [`program`] create and change records.
//!
//! A node joins the tree only through its parent's id. A node's lineage is its parent, its
//! parent's parent, and so on up to a top-level member. An action that needs a lineage takes it
//! as a list of nodes, checks that the list links up from the node and ends at a top-level
//! member, and counts depths from it, so no record stores a depth or a list of ancestors. The
//! kernel does keep running totals, such as a lane node's downline and children balances and a
//! lane's pool balance, so that no action loads descendants. The host never edits them.
//!
//! # Actions
//!
//! [`program`] lists every action. An action takes the records it needs by value, and returns
//! a receipt with the records it changed or created and the amounts the host must move. The
//! host stores every record in the receipt. A failed action returns an [`Error`] and no
//! records, so it changes nothing. The kernel panics only if the host breaks one of the duties
//! below.
//!
//! # Numbers
//!
//! Amounts are whole token base units. Indexes and kept rewards are fractional, in the host's
//! fixed-point type `N`, which implements [`FixedNumber`](mayf_fixed_number::FixedNumber).
//! Collecting also needs [`IntegralRounding`](mayf_fixed_number::IntegralRounding), to take
//! whole base units.
//!
//! Emission, indexes, rewards, gifts, and collections round down, so rounding never pays out
//! more than was emitted. Nothing is emitted while no lane has gauge votes, and emission that
//! reaches no balance, such as a lane's share while no one holds it, is never minted.
//!
//! Every balance and total fits a token supply, below 2^64 base units, and total emission stops
//! at `Amount::MAX`. Every factor the kernel applies to a balance is at most one: the override
//! rate times a depth, a sharing portion, and a lane's share of the gauge votes. Every index is
//! credits divided by a total of balances or votes, so no index exceeds total emission. So every
//! value the kernel computes stays below 2^64. [`create_protocol`](program::create_protocol)
//! therefore requires an `N` that holds every value below 2^64 on a grid at least as fine as
//! 2^-64, such as `U64F64` or `U192F64`. Each rounding step then moves less than one credit base
//! unit, and settling more often changes results by less than that per settlement.
//!
//! # The host
//!
//! The host operates the kernel, and the kernel trusts it. The host:
//! - authorizes each action, for example that only a node's owner sets its offer. Anyone may
//!   call [`settle_node_in_lane`](program::settle_node_in_lane).
//! - stores records by key, keeps node ids unique, and never deletes a lane node
//! - passes each lane node it has under its own node's id, from the action's lane
//! - passes a clock, `now`, that never goes back
//! - bounds each voter's gauge votes, for example by its voting power, so that the votes on
//!   all lanes sum to at most `Amount::MAX`
//! - deposits and withdraws whenever a member's AVA position in a lane changes, so the balances
//!   in a lane sum to at most the token's supply
//! - rejects a join whose fee is above the most the new member agreed to pay
//! - moves each join fee from the new member to the parent's owner, or to the protocol for a
//!   top-level seat, and mints collected credits

extern crate alloc;

mod emission;
mod error;
mod lane;
mod numbers;
pub mod program;
mod protocol;
mod tree;
mod weighting;

pub use emission::{EmissionRate, Emitter};
pub use error::{Error, Result};
pub use lane::Lane;
pub use mayf_micro_basis_points::MicroBasisPoints;
pub use numbers::{Amount, Seconds};
pub use protocol::Protocol;
pub use tree::{Depth, Identifiable, LaneNode, LanePool, NetworkTerms, Node, Offer};
pub use weighting::{Gauge, GaugeVote, LaneGauge};
