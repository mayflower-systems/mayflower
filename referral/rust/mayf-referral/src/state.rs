//! Validated permanent membership, admission policies, and scoped balances.

use crate::{ReferralError, ReferralPoints, ReferralRate, ReferralTreeConfig, Result};
use core::num::NonZeroU128;
use referral_tree::{DirectedRounding, FixedNumber};

/// Maximum number of nodes in a source-to-root path: the root plus seven
/// participating levels. This caps a network's `max_depth` at
/// `MAX_PATH_NODES - 2`, which is 6.
pub const MAX_PATH_NODES: usize = 8;
/// Maximum number of parent-child edges in a source-to-root path. Each edge
/// records the offer the child joined through.
pub const MAX_PATH_EDGES: usize = MAX_PATH_NODES - 1;

/// Persistence schema for [`ReferralNetwork`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferralNetworkSnapshot<I> {
    /// Network identifier. Every node, offer, and lane records it.
    pub id: I,
    /// Stored for the host. This crate does not check it.
    pub authority: I,
    /// The administrative root node. Every path ends here.
    pub root_node_id: I,
    /// Owner of the root node.
    pub root_owner: I,
    /// Deepest participating level, counting the root's children as level 0.
    /// A path holds at most `max_depth + 2` nodes. At most 6.
    pub max_depth: u8,
    /// The override rate is `override_numerator / override_denominator`.
    /// Validation requires `override_numerator < override_denominator` and
    /// `max_depth * override_numerator < override_denominator`.
    pub override_numerator: u32,
    /// See `override_numerator`.
    pub override_denominator: u32,
}

/// A validated network: its root and its fixed depth and override rate.
///
/// Build it with [`create_network`](crate::create_network) or from a
/// [`ReferralNetworkSnapshot`]. Getters return the fields that the snapshot
/// documents. The network does not track rewards or assets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferralNetwork<I> {
    pub(crate) id: I,
    pub(crate) authority: I,
    pub(crate) root_node_id: I,
    pub(crate) root_owner: I,
    pub(crate) max_depth: u8,
    pub(crate) override_numerator: u32,
    pub(crate) override_denominator: u32,
    pub(crate) override_rate: ReferralRate,
}

impl<I: Copy> ReferralNetwork<I> {
    pub fn id(&self) -> I {
        self.id
    }
    pub fn authority(&self) -> I {
        self.authority
    }
    pub fn root_node_id(&self) -> I {
        self.root_node_id
    }
    pub fn root_owner(&self) -> I {
        self.root_owner
    }
    /// The compact override rate, rounded up from the configured rational.
    pub const fn override_rate(&self) -> ReferralRate {
        self.override_rate
    }
    /// The network's concrete wide configuration for referral calculations.
    ///
    /// Widening the compact rate is exact. This configuration also supports
    /// point-time settlement over the entire `u64` token and clock range.
    pub fn config(&self) -> ReferralTreeConfig {
        ReferralTreeConfig {
            max_depth: self.max_depth,
            override_rate: ReferralPoints::from(self.override_rate),
        }
    }
    pub const fn max_depth(&self) -> u8 {
        self.max_depth
    }
}

impl<I> TryFrom<ReferralNetworkSnapshot<I>> for ReferralNetwork<I> {
    type Error = ReferralError;
    fn try_from(value: ReferralNetworkSnapshot<I>) -> Result<Self> {
        let override_rate = validate_network_config(
            value.max_depth,
            value.override_numerator,
            value.override_denominator,
        )?;
        Ok(Self {
            id: value.id,
            authority: value.authority,
            root_node_id: value.root_node_id,
            root_owner: value.root_owner,
            max_depth: value.max_depth,
            override_numerator: value.override_numerator,
            override_denominator: value.override_denominator,
            override_rate,
        })
    }
}

impl<I> From<ReferralNetwork<I>> for ReferralNetworkSnapshot<I> {
    fn from(value: ReferralNetwork<I>) -> Self {
        Self {
            id: value.id,
            authority: value.authority,
            root_node_id: value.root_node_id,
            root_owner: value.root_owner,
            max_depth: value.max_depth,
            override_numerator: value.override_numerator,
            override_denominator: value.override_denominator,
        }
    }
}

/// Check the depth and the outside override rate, then convert the rate to
/// [`ReferralRate`], rounding up, and validate it on the program's points grid.
///
/// Rounding up means upline members never get less than the stated rate, and a
/// small positive rate never becomes zero. The rounded value is checked again against
/// the depth bound, because rounding up can break it.
pub(crate) fn validate_network_config(
    max_depth: u8,
    numerator: u32,
    denominator: u32,
) -> Result<ReferralRate> {
    if usize::from(max_depth) + 2 > MAX_PATH_NODES
        || numerator >= denominator
        || u64::from(max_depth) * u64::from(numerator) >= u64::from(denominator)
    {
        return Err(ReferralError::InvalidConfiguration);
    }
    let denominator =
        NonZeroU128::new(u128::from(denominator)).ok_or(ReferralError::InvalidConfiguration)?;
    let override_rate = ReferralRate::checked_from_ratio_round(
        i128::from(numerator),
        denominator,
        DirectedRounding::Ceil,
    )
    .map_err(|_| ReferralError::InvalidConfiguration)?;
    let config = ReferralTreeConfig {
        max_depth,
        override_rate: ReferralPoints::from(override_rate),
    };
    referral_tree::validate_config(config).map_err(|_| ReferralError::InvalidConfiguration)?;
    Ok(override_rate)
}

/// Persistence schema for [`ReferralOffer`]. Validation requires a nonzero
/// `version`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferralOfferSnapshot<I> {
    /// Offer identifier.
    pub id: I,
    /// Network the offer belongs to.
    pub network_id: I,
    /// The node whose new direct children join through this offer.
    pub parent_node_id: I,
    /// Offer generation on the parent node, starting at 1. Publishing or
    /// replacing an offer takes the next version. Repricing keeps it.
    pub version: u64,
    /// False once the offer is closed or replaced.
    pub open: bool,
    /// Fee the next joiner pays, in the host's units.
    pub join_fee: u64,
    /// Amount added to `join_fee` after each successful join.
    pub join_fee_increment: u64,
}

/// A validated offer: the terms under which a parent admits new direct
/// children.
///
/// Getters return the fields that [`ReferralOfferSnapshot`] documents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferralOffer<I> {
    pub(crate) id: I,
    pub(crate) network_id: I,
    pub(crate) parent_node_id: I,
    pub(crate) version: u64,
    pub(crate) open: bool,
    pub(crate) join_fee: u64,
    pub(crate) join_fee_increment: u64,
}

impl<I: Copy> ReferralOffer<I> {
    pub fn id(&self) -> I {
        self.id
    }

    pub fn network_id(&self) -> I {
        self.network_id
    }

    pub fn parent_node_id(&self) -> I {
        self.parent_node_id
    }

    pub const fn version(&self) -> u64 {
        self.version
    }

    pub const fn is_open(&self) -> bool {
        self.open
    }

    pub const fn join_fee(&self) -> u64 {
        self.join_fee
    }

    pub const fn join_fee_increment(&self) -> u64 {
        self.join_fee_increment
    }
}

impl<I> TryFrom<ReferralOfferSnapshot<I>> for ReferralOffer<I> {
    type Error = ReferralError;

    fn try_from(snapshot: ReferralOfferSnapshot<I>) -> Result<Self> {
        if snapshot.version == 0 {
            return Err(ReferralError::InvalidOffer);
        }
        Ok(Self {
            id: snapshot.id,
            network_id: snapshot.network_id,
            parent_node_id: snapshot.parent_node_id,
            version: snapshot.version,
            open: snapshot.open,
            join_fee: snapshot.join_fee,
            join_fee_increment: snapshot.join_fee_increment,
        })
    }
}

impl<I> From<ReferralOffer<I>> for ReferralOfferSnapshot<I> {
    fn from(value: ReferralOffer<I>) -> Self {
        Self {
            id: value.id,
            network_id: value.network_id,
            parent_node_id: value.parent_node_id,
            version: value.version,
            open: value.open,
            join_fee: value.join_fee,
            join_fee_increment: value.join_fee_increment,
        }
    }
}

/// Persistence schema for [`ReferralNode`].
///
/// Path arrays run from the node to its root. Index 0 is the node itself, and
/// higher indexes walk toward the root. Edge `i` joins path nodes `i` and
/// `i + 1`.
///
/// Validation checks the shape of the record:
///
/// - `depth` is between 1 and [`MAX_PATH_NODES`].
/// - Exactly the first `depth` path entries are filled, and the first is this
///   node and its owner.
/// - No node appears twice in the path.
/// - Exactly the first `depth - 1` edges have an offer id and a nonzero
///   version.
/// - A node with `depth == MAX_PATH_NODES` has no active offer and a zero
///   `latest_offer_version`.
/// - An active offer requires a nonzero `latest_offer_version`.
///
/// Validation does not prove that the recorded offers existed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferralNodeSnapshot<I> {
    /// Node identifier.
    pub id: I,
    /// Network the node belongs to.
    pub network_id: I,
    /// The network's administrative root node.
    pub root_id: I,
    /// The member who owns this node. Only the owner may deposit or withdraw.
    pub owner: I,
    /// Number of nodes in the path, counting this node and the root. The root
    /// has depth 1, and members who join the root have depth 2.
    pub depth: u8,
    /// Count of successful direct joins under this node. It is not a list of
    /// children.
    pub joined_children: u64,
    /// This node, its parent, and so on up to the root, then `None`.
    pub path_node_ids: [Option<I>; MAX_PATH_NODES],
    /// Owner of each path node, recorded when this node joined.
    pub path_owners: [Option<I>; MAX_PATH_NODES],
    /// Offer accepted on each edge, then `None`. Kept for provenance. Rewards
    /// do not use it.
    pub path_offer_ids: [Option<I>; MAX_PATH_EDGES],
    /// Version of each accepted offer, then 0. Repricing does not change a
    /// version, so this does not record the fee paid.
    pub path_offer_versions: [u64; MAX_PATH_EDGES],
    /// The open offer through which new children can join, if any.
    pub active_offer_id: Option<I>,
    /// Version of the last offer this node published. 0 if it never published
    /// one.
    pub latest_offer_version: u64,
}

/// A validated permanent member: its source-to-root path and the offers
/// accepted on each edge.
///
/// No action moves or removes a node. Getters return the fields that
/// [`ReferralNodeSnapshot`] documents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferralNode<I> {
    pub(crate) id: I,
    pub(crate) network_id: I,
    pub(crate) root_id: I,
    pub(crate) owner: I,
    pub(crate) depth: u8,
    pub(crate) joined_children: u64,
    pub(crate) path_node_ids: [Option<I>; MAX_PATH_NODES],
    pub(crate) path_owners: [Option<I>; MAX_PATH_NODES],
    pub(crate) path_offer_ids: [Option<I>; MAX_PATH_EDGES],
    pub(crate) path_offer_versions: [u64; MAX_PATH_EDGES],
    pub(crate) active_offer_id: Option<I>,
    pub(crate) latest_offer_version: u64,
}

impl<I: Copy> ReferralNode<I> {
    pub const fn joined_children(&self) -> u64 {
        self.joined_children
    }
    pub fn id(&self) -> I {
        self.id
    }

    pub fn network_id(&self) -> I {
        self.network_id
    }

    pub fn root_id(&self) -> I {
        self.root_id
    }

    pub fn owner(&self) -> I {
        self.owner
    }

    /// Number of nodes in the path, counting this node and the root. The root
    /// has depth 1. This is not `referral_tree` depth.
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    pub fn path_node_ids(&self) -> [Option<I>; MAX_PATH_NODES] {
        self.path_node_ids
    }

    pub fn path_owners(&self) -> [Option<I>; MAX_PATH_NODES] {
        self.path_owners
    }

    pub fn path_offer_ids(&self) -> [Option<I>; MAX_PATH_EDGES] {
        self.path_offer_ids
    }

    pub const fn path_offer_versions(&self) -> [u64; MAX_PATH_EDGES] {
        self.path_offer_versions
    }

    pub fn active_offer_id(&self) -> Option<I> {
        self.active_offer_id
    }

    pub const fn latest_offer_version(&self) -> u64 {
        self.latest_offer_version
    }
}

impl<I: Copy + Eq> TryFrom<ReferralNodeSnapshot<I>> for ReferralNode<I> {
    type Error = ReferralError;

    fn try_from(snapshot: ReferralNodeSnapshot<I>) -> Result<Self> {
        let depth = usize::from(snapshot.depth);
        if depth == 0
            || depth > MAX_PATH_NODES
            || snapshot.path_node_ids[0] != Some(snapshot.id)
            || snapshot.path_owners[0] != Some(snapshot.owner)
            || (depth == MAX_PATH_NODES
                && (snapshot.active_offer_id.is_some() || snapshot.latest_offer_version != 0))
            || (snapshot.active_offer_id.is_some() && snapshot.latest_offer_version == 0)
        {
            return Err(ReferralError::InvalidNodePath);
        }
        for index in 0..MAX_PATH_NODES {
            let populated = index < depth;
            if snapshot.path_node_ids[index].is_some() != populated
                || snapshot.path_owners[index].is_some() != populated
            {
                return Err(ReferralError::InvalidNodePath);
            }
        }
        for left in 0..depth {
            for right in (left + 1)..depth {
                if snapshot.path_node_ids[left] == snapshot.path_node_ids[right] {
                    return Err(ReferralError::DuplicatePathNode);
                }
            }
        }

        let edge_count = depth - 1;
        for index in 0..MAX_PATH_EDGES {
            if index < edge_count {
                if snapshot.path_offer_ids[index].is_none()
                    || snapshot.path_offer_versions[index] == 0
                {
                    return Err(ReferralError::InvalidNodePath);
                }
            } else if snapshot.path_offer_ids[index].is_some()
                || snapshot.path_offer_versions[index] != 0
            {
                return Err(ReferralError::InvalidNodePath);
            }
        }

        Ok(Self {
            id: snapshot.id,
            network_id: snapshot.network_id,
            root_id: snapshot.root_id,
            owner: snapshot.owner,
            depth: snapshot.depth,
            joined_children: snapshot.joined_children,
            path_node_ids: snapshot.path_node_ids,
            path_owners: snapshot.path_owners,
            path_offer_ids: snapshot.path_offer_ids,
            path_offer_versions: snapshot.path_offer_versions,
            active_offer_id: snapshot.active_offer_id,
            latest_offer_version: snapshot.latest_offer_version,
        })
    }
}

impl<I> From<ReferralNode<I>> for ReferralNodeSnapshot<I> {
    fn from(value: ReferralNode<I>) -> Self {
        Self {
            id: value.id,
            network_id: value.network_id,
            root_id: value.root_id,
            owner: value.owner,
            depth: value.depth,
            joined_children: value.joined_children,
            path_node_ids: value.path_node_ids,
            path_owners: value.path_owners,
            path_offer_ids: value.path_offer_ids,
            path_offer_versions: value.path_offer_versions,
            active_offer_id: value.active_offer_id,
            latest_offer_version: value.latest_offer_version,
        }
    }
}

/// Persistence schema for [`ReferralLane`].
///
/// A scope is a domain of comparable units, such as one emission channel.
/// Balances from different scopes never mix. Validation requires
/// `subtree_balance >= direct_balance` and
/// `lifetime_deposited == direct_balance + lifetime_withdrawn`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferralLaneSnapshot<I> {
    /// Lane identifier. The host must allow at most one lane per node and
    /// scope.
    pub id: I,
    /// Network the lane belongs to.
    pub network_id: I,
    /// The node whose deposits this lane holds.
    pub node_id: I,
    /// The scope this lane's units belong to.
    pub scope_id: I,
    /// The node's owner, copied when the lane was created.
    pub owner: I,
    /// Units the node's owner has deposited and not withdrawn.
    pub direct_balance: u64,
    /// `direct_balance` plus the direct balances of the node's whole downline
    /// in this scope. On the root lane, this is the scope's total deposits.
    pub subtree_balance: u64,
    /// Total units the node's owner has ever deposited in this lane.
    pub lifetime_deposited: u128,
    /// Total units the node's owner has ever withdrawn from this lane.
    pub lifetime_withdrawn: u128,
}

/// A validated lane: one node's direct and subtree balances in one scope.
///
/// Lifetime counters count only the owner's own deposits and withdrawals.
/// Upline lanes see downline activity only in `subtree_balance`, so one
/// deposit is not counted once per level. Getters return the fields that
/// [`ReferralLaneSnapshot`] documents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferralLane<I> {
    pub(crate) id: I,
    pub(crate) network_id: I,
    pub(crate) node_id: I,
    pub(crate) scope_id: I,
    pub(crate) owner: I,
    pub(crate) direct_balance: u64,
    pub(crate) subtree_balance: u64,
    pub(crate) lifetime_deposited: u128,
    pub(crate) lifetime_withdrawn: u128,
}

impl<I: Copy> ReferralLane<I> {
    pub fn id(&self) -> I {
        self.id
    }

    pub fn network_id(&self) -> I {
        self.network_id
    }

    pub fn node_id(&self) -> I {
        self.node_id
    }

    pub fn scope_id(&self) -> I {
        self.scope_id
    }

    pub fn owner(&self) -> I {
        self.owner
    }

    pub const fn direct_balance(&self) -> u64 {
        self.direct_balance
    }

    pub const fn subtree_balance(&self) -> u64 {
        self.subtree_balance
    }

    pub const fn lifetime_deposited(&self) -> u128 {
        self.lifetime_deposited
    }

    pub const fn lifetime_withdrawn(&self) -> u128 {
        self.lifetime_withdrawn
    }
}

impl<I> TryFrom<ReferralLaneSnapshot<I>> for ReferralLane<I> {
    type Error = ReferralError;

    fn try_from(snapshot: ReferralLaneSnapshot<I>) -> Result<Self> {
        let expected_deposited = u128::from(snapshot.direct_balance)
            .checked_add(snapshot.lifetime_withdrawn)
            .ok_or(ReferralError::InvalidLaneAccounting)?;
        if snapshot.subtree_balance < snapshot.direct_balance
            || snapshot.lifetime_deposited != expected_deposited
        {
            return Err(ReferralError::InvalidLaneAccounting);
        }
        Ok(Self {
            id: snapshot.id,
            network_id: snapshot.network_id,
            node_id: snapshot.node_id,
            scope_id: snapshot.scope_id,
            owner: snapshot.owner,
            direct_balance: snapshot.direct_balance,
            subtree_balance: snapshot.subtree_balance,
            lifetime_deposited: snapshot.lifetime_deposited,
            lifetime_withdrawn: snapshot.lifetime_withdrawn,
        })
    }
}

impl<I> From<ReferralLane<I>> for ReferralLaneSnapshot<I> {
    fn from(value: ReferralLane<I>) -> Self {
        Self {
            id: value.id,
            network_id: value.network_id,
            node_id: value.node_id,
            scope_id: value.scope_id,
            owner: value.owner,
            direct_balance: value.direct_balance,
            subtree_balance: value.subtree_balance,
            lifetime_deposited: value.lifetime_deposited,
            lifetime_withdrawn: value.lifetime_withdrawn,
        }
    }
}
