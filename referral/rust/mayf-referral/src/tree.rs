//! The referral tree: membership, and how each lane's credits reach its members.
//!
//! The tree has no root node. A member either takes a top-level seat, for the protocol's
//! top-level offer, or joins directly under a parent, for the parent's offer. Either way it sets
//! its own offer for its future children. Top-level members are siblings, and each heads its
//! own branch.
//!
//! Each person has one [`Node`]. It holds what belongs to the member in every lane: its parent's
//! id, its offer, the portion of its credits it shares with its children, and the credits it
//! keeps. A node's lineage is its parent, its parent's parent, and so on up to a top-level
//! member. An action that needs a lineage takes it as a
//! list of nodes, checks that the list links up from the node to a top-level member, and counts
//! depths from it. No record stores a depth.
//!
//! The tree has the same shape in every lane: each node connects to its parent through all the
//! lanes at once, like a bundle of fibers. A [`LaneNode`] holds one node's balances in one lane,
//! and the indexes that turn them into credits. In each lane, a member earns credits for its
//! balance there. It pays a fixed override on its points to each member of its upline, and it
//! shares its sharing portion of what it earns with its direct children. It keeps the gifts it
//! receives whole, so gifts pass down one level and no further. The credits it keeps from every
//! lane gather in its node.
//!
//! A lane's pool turns the lane's allocation into credits per unit of balance: its reward
//! index. A lane node earns its core points times the reward index's growth, plus its balance
//! times the growth of its parent's gift index. An action settles one member and its lineage,
//! in one lane. Siblings and children never need loading, because what they are owed waits in
//! their parents' gift indexes.

mod lane_chain;
mod lane_node;
mod lane_pool;
mod node;
mod offer;
mod terms;

pub(crate) use lane_chain::LaneChain;
pub use lane_node::LaneNode;
pub use lane_pool::LanePool;
pub use node::Node;
pub use offer::Offer;
pub use terms::NetworkTerms;

/// A depth in the referral tree: a member's number of upline members. Top-level members have
/// depth 0.
pub type Depth = u8;

/// An identifier chosen by the host, such as an account address
pub trait Identifiable: Copy + Eq {}

impl<T: Copy + Eq> Identifiable for T {}
