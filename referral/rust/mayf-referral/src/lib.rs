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
//! Each lane receives its own cumulative earnings as data, without mixing asset units.
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
//! [`Protocol`] holds immutable tree terms and the lane registry. Each [`Lane`]
//! holds a reward index and its consumed earnings total. Each [`Node`] holds its
//! permanent parent, cached depth, family index, escrow and net accrued credits.
//! A [`LaneNode`] holds only that node's
//! personal and strict-descendant balances and source checkpoints in one asset.
//!
//! [`program::set_balance`] touches one lane and the member's ancestry. Family
//! rate and escrow changes settle all of the parent's source lanes at one
//! snapshot before changing terms. The permanent [`LaneMask`] makes omissions
//! detectable; zeroed positions must remain stored. No action scans descendants
//! or siblings. Path work scales with the configured depth, independently of
//! the network's population. The lane mask has capacity for sixteen lanes;
//! this storage limit is separate from tree height. Input vectors are reused
//! for positions and ancestry in receipts. The host must validate its chosen
//! height against execution limits.
//!
//! # Earnings and arithmetic
//!
//! Actions receive each lane's cumulative earnings as a precise fixed number
//! `N`. Single-lane actions accept `total_earned`; family actions pair each lane
//! with its own total. Earnings must be nonnegative and cannot decrease. A lane
//! distributes only the increase over its previously consumed total. Repeating
//! a total creates no new entitlement. Registration establishes a baseline so
//! earlier earnings cannot be claimed. There are no gauges, votes, schedules,
//! clocks, or emission sources in this library.
//!
//! Whole balances use [`Amount`]. Indexes and fractional claims use a checked
//! [`mayf_fixed_number::FixedNumber`]. Use a wide driver such as `U192F64` for
//! cumulative accounting and rate intermediates. The host chooses lifetime
//! bounds; arithmetic failure returns an error rather than wrapping or capping
//! earnings. Cumulative totals need not fit in `u64`. Collection floors once to
//! whole token units and preserves the rest.
//! Division and rational-rate carries remove settlement-frequency dependence
//! while eligibility is unchanged. Actual asset changes commit and close the
//! affected lane's remainder; family term changes commit and close every source
//! lane's remainder. Family-index and marketing carries close at the same
//! boundary. This prevents old lane dust arriving under new terms.
//! Shared residuals become explicit unallocated budget, never a newcomer's
//! claim. These operations write shared records and therefore contend across
//! branches; bounded memory does not imply unconstrained transaction throughput.
//! An empty lane still consumes its earnings as unallocated, with no later backpay.
//!
//! For grid quantum `Q`, a lane or family index remainder is less than
//! its integer denominator times `Q`; a marketing-rate carry is less than `Q`
//! credits. An actual asset change closes less than `Q*lane_balance`.
//! A family change closes less than
//! `Q*(sum(source_lane_balances) + child_escrow + 1)`.
//! At sixteen lanes, u64 totals and Q <= 2^-64, these bounds are below 1 and
//! 17 credit base units respectively. No-op settlement
//! closes nothing. Unrelated members of a closed source lane also forgo its
//! unattributed dust; already-attributed individual fractions remain owned.
//!
//! # Host contract
//!
//! The host authenticates records and authorizes the owner-controlled actions.
//! It supplies unique node IDs, binds every lane position to its canonical
//! `(network, lane, node)` key, and preserves all fields between actions. It
//! supplies canonical cumulative earnings for each lane, authenticating their
//! source and preventing the same budget from being assigned twice. Before any
//! balance or family eligibility change, every affected total must be current
//! through that change. Monotonicity cannot prove freshness: the host must
//! establish the common settlement boundary. Read-only projections are safe only
//! when subsequent canonical mutations settle old eligibility first.
//!
//! Eligible asset balances may be internal positions; no token transfer is
//! assumed. The host atomically persists each receipt and its effects: position
//! changes, ordinary credit fees to parents, root-auction burns, escrow transfers,
//! and minting collected **net** rewards. Escrow is exclusively held and cannot
//! simultaneously fund other uses, other families, or bids. Root awards
//! come from the protocol auction and are consumed once. Bootstrap credits for
//! the first auction are external to the lane earnings tracked here.
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

mod error;
mod lane;
mod lanes;
mod numbers;
pub mod program;
mod protocol;
mod tree;

pub use error::{Error, Result};
pub use lane::Lane;
pub use lanes::{LaneId, LaneMask, MAX_LANES};
pub use mayf_micro_basis_points::MicroBasisPoints;
pub use numbers::{Amount, Change};
pub use protocol::Protocol;
pub use tree::{Depth, Identifiable, LaneNode, LanePool, NetworkTerms, Node, Offer, RootSeat};
