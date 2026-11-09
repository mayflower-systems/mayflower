#![no_std]
#![forbid(unsafe_code)]

//! Pure referral accounting: validated actions in [`program`], plain state, and
//! receipts. There is no I/O, serialization, account allocation, or token mint here.
//!
//! # Allocation
//!
//! A source at depth `d` keeps `1 - d*q` of its stake-weighted lane allocation;
//! each earning ancestor receives `q`. The logical protocol root earns nothing.
//! Top seats have depth zero. The configured maximum depth `H` permits at most
//! `H + 1` earning members per path; it is not fixed to a particular deployment.
//! Lane allocations follow proportional gauge weights, without mixing asset units.
//! Network terms require `H*q < 1`, preserving a positive own-asset share.
//! Zero depth and zero commission are valid configurations. The host chooses
//! the network terms; parents cannot change them through ordinary actions.
//!
//! A parent markets only its gross overrides: it shares portion `sharing` among
//! immediate children in proportion to their escrowed, already-issued credits.
//! Own income and incoming gifts stay with the node. Gifts are terminal. With no
//! child escrow, all overrides stay with the parent. Escrow never auto-compounds.
//!
//! # Lazy state
//!
//! [`Protocol`] holds immutable tree terms and the gauge. Each [`Lane`] holds a
//! reward index. Each [`Node`] holds its permanent parent, cached depth, family
//! index, escrow and net accrued credits. A [`LaneNode`] holds only that node's
//! personal and strict-descendant balances and source checkpoints in one asset.
//!
//! [`program::set_balance`] touches one lane and the member's ancestry. Family
//! rate and escrow changes settle all of the parent's source lanes at one
//! snapshot before changing terms. The permanent [`LaneMask`] makes omissions
//! detectable; zeroed positions must remain stored. No action scans descendants
//! or siblings. Path work scales with the configured depth, independently of
//! the network's population. The lane mask has capacity for sixteen lanes;
//! this storage limit is separate from tree height. Input vectors are reused
//! in receipts. The host must validate its chosen height against execution limits.
//!
//! # Emission and arithmetic
//!
//! [`Emission`] is the authenticated cumulative integral of the host's emission
//! schedule. Its type is the accounting number `N`, not a token amount;
//! lifetime emission does not saturate at `u64::MAX`.
//! Mayflower's planned schedule declines continuously, but the referral kernel
//! neither selects nor enforces a decay curve or its parameters.
//!
//! Whole balances use [`Amount`]. Indexes and fractional claims use a checked
//! [`mayf_fixed_number::FixedNumber`]. Use a wide driver such as `U192F64` for
//! cumulative accounting and rate intermediates. The host chooses lifetime
//! bounds; arithmetic failure returns an error rather than wrapping or capping
//! emission. Collection floors once to whole token units and preserves the rest.
//! Division and rational-rate carries remove settlement-frequency dependence
//! while eligibility is unchanged. Actual asset changes commit and close the
//! gauge's shared remainder; family term changes also commit and close every
//! source lane's remainder. Family-index and marketing carries close at the
//! same boundary. This prevents upstream dust arriving under new terms.
//! Shared residuals become explicit unallocated budget, never a newcomer's
//! claim. These operations write shared records and therefore contend across
//! branches; bounded memory does not imply unconstrained transaction throughput.
//! Empty-lane and zero-vote allocation is unissued, with no later backpay.
//!
//! For grid quantum `Q`, a gauge, lane, or family index remainder is less than
//! its integer denominator times `Q`; a marketing-rate carry is less than `Q`
//! credits. An actual asset change closes less than `Q*(votes + lane_balance)`.
//! A family change closes less than
//! `Q*(votes + sum(source_lane_balances) + child_escrow + 1)`.
//! At sixteen lanes, u64 totals and Q <= 2^-64, these bounds are below 2 and
//! 18 credit base units respectively. No-op settlement
//! closes nothing. Unrelated members of a closed source lane also forgo its
//! unattributed dust; already-attributed individual fractions remain owned.
//!
//! # Host contract
//!
//! The host authenticates records and authorizes the owner-controlled actions.
//! It supplies unique node IDs, binds every lane position to its canonical
//! `(network, lane, node)` key, and preserves all fields between actions. It
//! supplies one canonical emission total at the current monotone clock, rather
//! than accepting a caller's claimed total. Read-only projections are safe only
//! when subsequent canonical mutations settle the old interval first.
//!
//! The host atomically persists each receipt and its custody effects: asset
//! changes, ordinary credit fees to parents, root-auction burns, escrow transfers,
//! and minting collected **net** rewards. Escrow is exclusively held and cannot
//! simultaneously fund gauges, clearing, other families, or bids. Root awards
//! come from the protocol auction and are consumed once. Bootstrap credits for
//! the first auction are external to the emission allocation tracked here.
//!
//! Solana adapters should deserialize bounded vectors directly into heap storage
//! or borrow account data; never build a whole lineage of records on the stack.
//! Dispatch large actions through separate non-inlined instruction handlers:
//! keeping several owned argument/receipt sets in one frame can exceed 4 KiB.
//! The default bump allocator does not reclaim dropped buffers during a call,
//! so reuse returned vectors when composing actions. Budget compute for the
//! complete host transaction, including decoding, persistence, and custody CPIs.
//! This crate guarantees bounded work, not a particular host's compute budget.

extern crate alloc;

mod emission;
mod error;
mod lane;
mod lanes;
mod numbers;
pub mod program;
mod protocol;
mod tree;
mod weighting;

pub use emission::Emission;
pub use error::{Error, Result};
pub use lane::Lane;
pub use lanes::{LaneId, LaneMask, MAX_LANES};
pub use mayf_micro_basis_points::MicroBasisPoints;
pub use numbers::{Amount, Change, Seconds};
pub use protocol::Protocol;
pub use tree::{Depth, Identifiable, LaneNode, LanePool, NetworkTerms, Node, Offer, RootSeat};
pub use weighting::{Gauge, GaugeVote, LaneGauge};
