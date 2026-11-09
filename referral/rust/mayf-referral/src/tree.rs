//! Permanent membership and lazy accounting. Depth zero is an auctioned seat
//! under the non-earning protocol root; the network chooses its maximum depth.
//! A lane position stores personal and strict-descendant balances. Family
//! escrow and the gift index are shared by all lanes and live on the node.

mod family;
mod lane_chain;
mod lane_node;
mod lane_pool;
mod node;
mod offer;
mod root_seat;
mod terms;

pub(crate) use family::{close_sources, settle_sources};
pub(crate) use lane_chain::LaneChain;
pub use lane_node::LaneNode;
pub use lane_pool::LanePool;
pub use node::Node;
pub use offer::Offer;
pub use root_seat::RootSeat;
pub use terms::NetworkTerms;

/// Number of earning ancestors. Top-level seats have depth zero.
/// The network configures its maximum within this representation's `0..=255`.
pub type Depth = u8;

/// A host storage key, such as a Solana address.
pub trait Identifiable: Copy + Eq {}
impl<T: Copy + Eq> Identifiable for T {}
