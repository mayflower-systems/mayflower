//! Pure referral state transitions and their side-effect receipts.

use crate::{
    MAX_PATH_EDGES, MAX_PATH_NODES, ReferralError, ReferralLane, ReferralNetwork, ReferralNode,
    ReferralOffer, Result,
};

/// An admission fee for the host to execute in the same atomic step that
/// stores the returned records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeeTransfer<I> {
    /// The joining member.
    pub payer: I,
    /// Owner of the parent node.
    pub recipient: I,
    /// Fee in the host's chosen units. May be zero.
    pub amount: u64,
}

/// Arguments to [`create_network`]. The fields match
/// [`ReferralNetworkSnapshot`](crate::ReferralNetworkSnapshot).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreateNetworkArgs<I> {
    /// New network's identifier.
    pub network_id: I,
    /// Stored for the host. This crate does not check it.
    pub authority: I,
    /// Identifier of the administrative root node.
    pub node_id: I,
    /// Owner of the root node.
    pub owner: I,
    /// Deepest participating level, counting the root's children as level 0.
    /// At most 6.
    pub max_depth: u8,
    /// The override rate is `override_numerator / override_denominator`.
    pub override_numerator: u32,
    /// See `override_numerator`.
    pub override_denominator: u32,
}

/// Records to store after [`create_network`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreateNetworkReceipt<I> {
    /// The new network.
    pub network: ReferralNetwork<I>,
    /// The administrative root node, at depth 1 with no offer.
    pub node: ReferralNode<I>,
}

/// Create a network and its administrative root node.
///
/// Members join below the root. Its direct children are the top members of
/// the participating tree. The override rate is converted to
/// [`ReferralRate`](crate::ReferralRate), rounding up, then checked against
/// `max_depth` on the program's points grid.
///
/// Errors: [`ReferralError::InvalidConfiguration`] if `max_depth > 6`,
/// the ratio is not below 1, or `max_depth * ratio >= 1`.
pub fn create_network<I: Copy>(args: CreateNetworkArgs<I>) -> Result<CreateNetworkReceipt<I>> {
    let override_rate = crate::state::validate_network_config(
        args.max_depth,
        args.override_numerator,
        args.override_denominator,
    )?;
    let network_id = args.network_id;
    let mut path_node_ids = [None; MAX_PATH_NODES];
    let mut path_owners = [None; MAX_PATH_NODES];
    path_node_ids[0] = Some(args.node_id);
    path_owners[0] = Some(args.owner);
    Ok(CreateNetworkReceipt {
        network: ReferralNetwork {
            id: network_id,
            authority: args.authority,
            root_node_id: args.node_id,
            root_owner: args.owner,
            max_depth: args.max_depth,
            override_numerator: args.override_numerator,
            override_denominator: args.override_denominator,
            override_rate,
        },
        node: ReferralNode {
            id: args.node_id,
            network_id,
            root_id: args.node_id,
            owner: args.owner,
            depth: 1,
            joined_children: 0,
            path_node_ids,
            path_owners,
            path_offer_ids: [None; MAX_PATH_EDGES],
            path_offer_versions: [0; MAX_PATH_EDGES],
            active_offer_id: None,
            latest_offer_version: 0,
        },
    })
}

/// Arguments to [`publish_offer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublishOfferArgs<I> {
    /// The network the parent belongs to.
    pub network: ReferralNetwork<I>,
    /// The node that will admit children through the offer.
    pub parent_node: ReferralNode<I>,
    /// New offer's identifier.
    pub offer_id: I,
    /// Fee the first joiner pays.
    pub join_fee: u64,
    /// Amount added to the fee after each successful join.
    pub join_fee_increment: u64,
}

/// Records to store after [`publish_offer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublishOfferReceipt<I> {
    /// The parent, with the new offer active and its version recorded.
    pub next_parent_node: ReferralNode<I>,
    /// The new open offer.
    pub offer: ReferralOffer<I>,
}

/// Open an offer through which new members can join directly under
/// `parent_node`.
///
/// The offer takes version `latest_offer_version + 1` and becomes the parent's
/// active offer.
///
/// Errors: [`ReferralError::MaximumDepth`] if the parent is at the deepest
/// level; [`ReferralError::ActiveOfferExists`] if the parent already has an
/// active offer; and the errors of [`validate_node_network`].
pub fn publish_offer<I: Copy + Eq>(args: PublishOfferArgs<I>) -> Result<PublishOfferReceipt<I>> {
    validate_node_network(&args.network, &args.parent_node)?;
    if usize::from(args.parent_node.depth) >= usize::from(args.network.max_depth) + 2 {
        return Err(ReferralError::MaximumDepth);
    }
    if args.parent_node.active_offer_id.is_some() {
        return Err(ReferralError::ActiveOfferExists);
    }
    let version = args
        .parent_node
        .latest_offer_version
        .checked_add(1)
        .ok_or(ReferralError::ArithmeticFailure)?;
    Ok(PublishOfferReceipt {
        next_parent_node: ReferralNode {
            active_offer_id: Some(args.offer_id),
            latest_offer_version: version,
            ..args.parent_node
        },
        offer: ReferralOffer {
            id: args.offer_id,
            network_id: args.network.id,
            parent_node_id: args.parent_node.id,
            version,
            open: true,
            join_fee: args.join_fee,
            join_fee_increment: args.join_fee_increment,
        },
    })
}

/// Arguments to [`update_offer_pricing`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpdateOfferPricingArgs<I> {
    /// The network the parent belongs to.
    pub network: ReferralNetwork<I>,
    /// The node that owns the offer.
    pub parent_node: ReferralNode<I>,
    /// The parent's active offer.
    pub offer: ReferralOffer<I>,
    /// New fee for the next joiner. `None` keeps the current fee.
    pub join_fee: Option<u64>,
    /// New increment. `None` keeps the current increment.
    pub join_fee_increment: Option<u64>,
}

/// Change the fee or increment of an active offer for future joins.
///
/// Returns the updated offer to store. The version does not change, and
/// members who already joined are unaffected.
///
/// Errors: the offer is not the parent's open, current offer
/// ([`ReferralError::OfferClosed`], [`ReferralError::NoActiveOffer`],
/// [`ReferralError::OfferVersionMismatch`],
/// [`ReferralError::OfferParentMismatch`]), or the errors of
/// [`validate_node_network`].
pub fn update_offer_pricing<I: Copy + Eq>(
    args: UpdateOfferPricingArgs<I>,
) -> Result<ReferralOffer<I>> {
    validate_active_offer(&args.network, &args.parent_node, &args.offer)?;
    Ok(ReferralOffer {
        join_fee: args.join_fee.unwrap_or(args.offer.join_fee),
        join_fee_increment: args
            .join_fee_increment
            .unwrap_or(args.offer.join_fee_increment),
        ..args.offer
    })
}

/// Arguments to [`close_offer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CloseOfferArgs<I> {
    /// The network the parent belongs to.
    pub network: ReferralNetwork<I>,
    /// The node that owns the offer.
    pub parent_node: ReferralNode<I>,
    /// The parent's active offer.
    pub offer: ReferralOffer<I>,
}

/// Records to store after [`close_offer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CloseOfferReceipt<I> {
    /// The parent, with no active offer.
    pub next_parent_node: ReferralNode<I>,
    /// The offer, now closed.
    pub closed_offer: ReferralOffer<I>,
}

/// Close the parent's active offer.
///
/// The parent can publish a new offer later, which takes the next version.
///
/// Errors: the same as [`update_offer_pricing`].
pub fn close_offer<I: Copy + Eq>(args: CloseOfferArgs<I>) -> Result<CloseOfferReceipt<I>> {
    validate_active_offer(&args.network, &args.parent_node, &args.offer)?;
    Ok(CloseOfferReceipt {
        next_parent_node: ReferralNode {
            active_offer_id: None,
            ..args.parent_node
        },
        closed_offer: ReferralOffer {
            open: false,
            ..args.offer
        },
    })
}

/// Arguments to [`replace_offer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplaceOfferArgs<I> {
    /// The network the parent belongs to.
    pub network: ReferralNetwork<I>,
    /// The node that owns the offer.
    pub parent_node: ReferralNode<I>,
    /// The parent's active offer, which will be closed.
    pub active_offer: ReferralOffer<I>,
    /// Identifier of the new offer. Must differ from the active offer's.
    pub replacement_offer_id: I,
    /// Fee the first joiner of the new offer pays.
    pub join_fee: u64,
    /// Amount added to the new offer's fee after each successful join.
    pub join_fee_increment: u64,
}

/// Records to store after [`replace_offer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplaceOfferReceipt<I> {
    /// The parent, with the new offer active and its version recorded.
    pub next_parent_node: ReferralNode<I>,
    /// The old offer, now closed.
    pub closed_offer: ReferralOffer<I>,
    /// The new open offer.
    pub replacement_offer: ReferralOffer<I>,
}

/// Close the parent's active offer and open a new one in one step.
///
/// The new offer takes version `latest_offer_version + 1`.
///
/// Errors: [`ReferralError::OfferIdReused`] if the new id equals the active
/// offer's id, and the errors of [`update_offer_pricing`].
pub fn replace_offer<I: Copy + Eq>(args: ReplaceOfferArgs<I>) -> Result<ReplaceOfferReceipt<I>> {
    validate_active_offer(&args.network, &args.parent_node, &args.active_offer)?;
    if args.replacement_offer_id == args.active_offer.id {
        return Err(ReferralError::OfferIdReused);
    }
    let version = args
        .parent_node
        .latest_offer_version
        .checked_add(1)
        .ok_or(ReferralError::ArithmeticFailure)?;
    Ok(ReplaceOfferReceipt {
        next_parent_node: ReferralNode {
            active_offer_id: Some(args.replacement_offer_id),
            latest_offer_version: version,
            ..args.parent_node
        },
        closed_offer: ReferralOffer {
            open: false,
            ..args.active_offer
        },
        replacement_offer: ReferralOffer {
            id: args.replacement_offer_id,
            network_id: args.network.id,
            parent_node_id: args.parent_node.id,
            version,
            open: true,
            join_fee: args.join_fee,
            join_fee_increment: args.join_fee_increment,
        },
    })
}

/// Arguments to [`join`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JoinArgs<I> {
    /// Highest fee the joiner accepts. If the price rose after the joiner saw
    /// it, the join fails instead of overcharging.
    pub max_join_fee: u64,
    /// The network the parent belongs to.
    pub network: ReferralNetwork<I>,
    /// The node to join under.
    pub parent_node: ReferralNode<I>,
    /// The parent's active offer.
    pub offer: ReferralOffer<I>,
    /// Identifier of the new node. The host must keep it unique; `join` only
    /// checks that it is not already on the parent's path.
    pub node_id: I,
    /// The joining member. It pays the fee and owns the new node.
    pub owner: I,
}

/// Records to store, and the fee to execute, after [`join`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JoinReceipt<I> {
    /// The offer, with its fee raised by the increment.
    pub next_offer: ReferralOffer<I>,
    /// The parent, with `joined_children` increased by one.
    pub next_parent_node: ReferralNode<I>,
    /// The new permanent node.
    pub node: ReferralNode<I>,
    /// The current fee, from the joiner to the parent's owner.
    pub join_fee_transfer: FeeTransfer<I>,
}

/// Admit a new member directly under `parent_node` through its active offer.
///
/// The join charges the offer's current fee, then raises the fee by the
/// increment for the next joiner. The new node's path is the node itself
/// followed by the parent's path. Its edge history is this offer's id and
/// version followed by the parent's edge history. The host must store all
/// three returned records and execute `join_fee_transfer` atomically.
///
/// Errors: [`ReferralError::MaximumDepth`] if the parent is at the deepest
/// level; [`ReferralError::JoinFeeExceeded`] if the fee is above
/// `max_join_fee`; [`ReferralError::DuplicatePathNode`] if `node_id` is on the
/// parent's path; and the errors of [`update_offer_pricing`].
#[inline(never)]
pub fn join<I: Copy + Eq>(args: &JoinArgs<I>) -> Result<JoinReceipt<I>> {
    let parent_depth = usize::from(args.parent_node.depth);
    if parent_depth >= usize::from(args.network.max_depth) + 2 {
        return Err(ReferralError::MaximumDepth);
    }
    validate_active_offer(&args.network, &args.parent_node, &args.offer)?;
    let price = args.offer.join_fee;
    if price > args.max_join_fee {
        return Err(ReferralError::JoinFeeExceeded);
    }
    let next_price = price
        .checked_add(args.offer.join_fee_increment)
        .ok_or(ReferralError::ArithmeticFailure)?;
    let joined_children = args
        .parent_node
        .joined_children
        .checked_add(1)
        .ok_or(ReferralError::ArithmeticFailure)?;
    for node_id in args.parent_node.path_node_ids[..parent_depth]
        .iter()
        .flatten()
    {
        if *node_id == args.node_id {
            return Err(ReferralError::DuplicatePathNode);
        }
    }
    let mut path_node_ids = [None; MAX_PATH_NODES];
    let mut path_owners = [None; MAX_PATH_NODES];
    let mut path_offer_ids = [None; MAX_PATH_EDGES];
    let mut path_offer_versions = [0; MAX_PATH_EDGES];
    path_node_ids[0] = Some(args.node_id);
    path_owners[0] = Some(args.owner);
    path_node_ids[1..=parent_depth]
        .copy_from_slice(&args.parent_node.path_node_ids[..parent_depth]);
    path_owners[1..=parent_depth].copy_from_slice(&args.parent_node.path_owners[..parent_depth]);
    path_offer_ids[0] = Some(args.offer.id);
    path_offer_versions[0] = args.offer.version;
    let inherited_edges = parent_depth - 1;
    if inherited_edges > 0 {
        path_offer_ids[1..=inherited_edges]
            .copy_from_slice(&args.parent_node.path_offer_ids[..inherited_edges]);
        path_offer_versions[1..=inherited_edges]
            .copy_from_slice(&args.parent_node.path_offer_versions[..inherited_edges]);
    }

    Ok(JoinReceipt {
        next_offer: ReferralOffer {
            join_fee: next_price,
            ..args.offer
        },
        next_parent_node: ReferralNode {
            joined_children,
            ..args.parent_node
        },
        node: ReferralNode {
            id: args.node_id,
            network_id: args.network.id,
            root_id: args.parent_node.root_id,
            owner: args.owner,
            depth: args.parent_node.depth + 1,
            joined_children: 0,
            path_node_ids,
            path_owners,
            path_offer_ids,
            path_offer_versions,
            active_offer_id: None,
            latest_offer_version: 0,
        },
        join_fee_transfer: FeeTransfer {
            payer: args.owner,
            recipient: args.parent_node.owner,
            amount: price,
        },
    })
}

/// Arguments to [`create_lane`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreateLaneArgs<I> {
    /// The network the node belongs to.
    pub network: ReferralNetwork<I>,
    /// The node that will own the lane.
    pub node: ReferralNode<I>,
    /// New lane's identifier.
    pub lane_id: I,
    /// Domain of comparable units, such as one emission channel.
    pub scope_id: I,
}

/// Record to store after [`create_lane`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreateLaneReceipt<I> {
    /// The new lane, with zero balances.
    pub lane: ReferralLane<I>,
}

/// Create an empty lane for one node in one scope.
///
/// The lane's owner is the node's owner. This function does not check for an
/// existing lane, so the host must allow at most one lane per node and scope.
///
/// Errors: the errors of [`validate_node_network`].
pub fn create_lane<I: Copy + Eq>(args: CreateLaneArgs<I>) -> Result<CreateLaneReceipt<I>> {
    validate_node_network(&args.network, &args.node)?;
    Ok(CreateLaneReceipt {
        lane: ReferralLane {
            id: args.lane_id,
            network_id: args.network.id,
            node_id: args.node.id,
            scope_id: args.scope_id,
            owner: args.node.owner,
            direct_balance: 0,
            subtree_balance: 0,
            lifetime_deposited: 0,
            lifetime_withdrawn: 0,
        },
    })
}

/// Arguments to [`deposit_into_lane`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DepositIntoLaneArgs<I> {
    /// The network the node belongs to.
    pub network: ReferralNetwork<I>,
    /// Source node whose direct balance increases.
    pub node: ReferralNode<I>,
    /// Expected balance domain for every supplied lane.
    pub scope_id: I,
    /// Must equal the source node's permanent owner.
    pub signer: I,
    /// Exact source-to-root lane path, followed by `None` entries.
    pub lanes: [Option<ReferralLane<I>>; MAX_PATH_NODES],
    /// Units to add. Must be positive.
    pub amount: u64,
}

/// Records to store after [`deposit_into_lane`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DepositIntoLaneReceipt<I> {
    /// The updated lane path, source first. Store every lane atomically.
    pub next_lanes: [Option<ReferralLane<I>>; MAX_PATH_NODES],
}

/// Add units to the source's `direct_balance` and to the `subtree_balance` of
/// every lane on its path, including its own.
///
/// Only the source lane records the deposit in its lifetime counter. The root
/// may deposit too; applications such as Mayflower Reserve may forbid that.
///
/// Errors: [`ReferralError::LaneDepositTooSmall`] if `amount` is zero;
/// [`ReferralError::LaneOwnerMismatch`] if `signer` is not the node's owner;
/// [`ReferralError::ArithmeticFailure`] on overflow; and the errors of
/// [`validate_lane_path`].
pub fn deposit_into_lane<I: Copy + Eq>(
    args: DepositIntoLaneArgs<I>,
) -> Result<DepositIntoLaneReceipt<I>> {
    if args.amount == 0 {
        return Err(ReferralError::LaneDepositTooSmall);
    }
    validate_lane_owner(&args.node, args.signer)?;
    validate_lane_path(&args.network, &args.node, args.scope_id, &args.lanes)?;

    let depth = usize::from(args.node.depth);
    let mut next_lanes = args.lanes;
    for maybe_lane in &mut next_lanes[..depth] {
        let lane = maybe_lane.as_mut().ok_or(ReferralError::LanePathMismatch)?;
        lane.subtree_balance = lane
            .subtree_balance
            .checked_add(args.amount)
            .ok_or(ReferralError::ArithmeticFailure)?;
    }
    let source_lane = next_lanes[0]
        .as_mut()
        .ok_or(ReferralError::LanePathMismatch)?;
    source_lane.direct_balance = source_lane
        .direct_balance
        .checked_add(args.amount)
        .ok_or(ReferralError::ArithmeticFailure)?;
    source_lane.lifetime_deposited = source_lane
        .lifetime_deposited
        .checked_add(u128::from(args.amount))
        .ok_or(ReferralError::ArithmeticFailure)?;

    Ok(DepositIntoLaneReceipt { next_lanes })
}

/// Arguments to [`withdraw_from_lane`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WithdrawFromLaneArgs<I> {
    /// The network the node belongs to.
    pub network: ReferralNetwork<I>,
    /// Source node whose direct balance decreases.
    pub node: ReferralNode<I>,
    /// Expected balance domain for every supplied lane.
    pub scope_id: I,
    /// Must equal the source node's permanent owner.
    pub signer: I,
    /// Exact source-to-root lane path, followed by `None` entries.
    pub lanes: [Option<ReferralLane<I>>; MAX_PATH_NODES],
    /// Units to remove. Must be positive and no more than the source's
    /// `direct_balance`.
    pub amount: u64,
}

/// Records to store after [`withdraw_from_lane`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WithdrawFromLaneReceipt<I> {
    /// The updated lane path, source first. Store every lane atomically.
    pub next_lanes: [Option<ReferralLane<I>>; MAX_PATH_NODES],
}

/// Remove units from the source's `direct_balance` and from the
/// `subtree_balance` of every lane on its path, including its own.
///
/// The limit is the source's own `direct_balance`, so an upline member cannot
/// withdraw a downline member's units.
///
/// Errors: [`ReferralError::LaneWithdrawalTooSmall`] if `amount` is zero;
/// [`ReferralError::LaneWithdrawalExceedsDirectBalance`];
/// [`ReferralError::LaneOwnerMismatch`] if `signer` is not the node's owner;
/// and the errors of [`validate_lane_path`].
pub fn withdraw_from_lane<I: Copy + Eq>(
    args: WithdrawFromLaneArgs<I>,
) -> Result<WithdrawFromLaneReceipt<I>> {
    if args.amount == 0 {
        return Err(ReferralError::LaneWithdrawalTooSmall);
    }
    validate_lane_owner(&args.node, args.signer)?;
    validate_lane_path(&args.network, &args.node, args.scope_id, &args.lanes)?;
    let source_direct_balance = args.lanes[0]
        .as_ref()
        .ok_or(ReferralError::LanePathMismatch)?
        .direct_balance;
    if args.amount > source_direct_balance {
        return Err(ReferralError::LaneWithdrawalExceedsDirectBalance);
    }

    let depth = usize::from(args.node.depth);
    let mut next_lanes = args.lanes;
    for maybe_lane in &mut next_lanes[..depth] {
        let lane = maybe_lane.as_mut().ok_or(ReferralError::LanePathMismatch)?;
        lane.subtree_balance = lane
            .subtree_balance
            .checked_sub(args.amount)
            .ok_or(ReferralError::InvalidLaneAccounting)?;
    }
    let source_lane = next_lanes[0]
        .as_mut()
        .ok_or(ReferralError::LanePathMismatch)?;
    source_lane.direct_balance -= args.amount;
    source_lane.lifetime_withdrawn = source_lane
        .lifetime_withdrawn
        .checked_add(u128::from(args.amount))
        .ok_or(ReferralError::ArithmeticFailure)?;

    Ok(WithdrawFromLaneReceipt { next_lanes })
}

/// Check that `node` belongs to `network`.
///
/// The node must carry the network's id, be no deeper than the network allows,
/// and have a path that ends at the network's root node and root owner.
///
/// Errors: [`ReferralError::NetworkMismatch`],
/// [`ReferralError::MaximumDepth`], or [`ReferralError::RootNodeMismatch`].
pub fn validate_node_network<I: Eq>(
    network: &ReferralNetwork<I>,
    node: &ReferralNode<I>,
) -> Result<()> {
    if node.network_id != network.id {
        return Err(ReferralError::NetworkMismatch);
    }
    if usize::from(node.depth) > usize::from(network.max_depth) + 2 {
        return Err(ReferralError::MaximumDepth);
    }
    let root_index = usize::from(node.depth) - 1;
    if node.root_id != network.root_node_id
        || node.path_node_ids[root_index].as_ref() != Some(&network.root_node_id)
        || node.path_owners[root_index].as_ref() != Some(&network.root_owner)
    {
        return Err(ReferralError::RootNodeMismatch);
    }
    Ok(())
}

fn validate_lane_owner<I: Eq>(node: &ReferralNode<I>, signer: I) -> Result<()> {
    if node.owner != signer {
        return Err(ReferralError::LaneOwnerMismatch);
    }
    Ok(())
}

/// Check that `lanes` is the exact source-to-root lane path of `node` in scope
/// `scope_id`.
///
/// There must be one lane per path node, in path order, then `None`. Each lane
/// must match its path node's id and owner, and the network and scope. No lane
/// id may appear twice.
///
/// Errors: [`ReferralError::LanePathMismatch`],
/// [`ReferralError::LaneScopeMismatch`], [`ReferralError::NetworkMismatch`],
/// and the errors of [`validate_node_network`].
pub fn validate_lane_path<I: Copy + Eq>(
    network: &ReferralNetwork<I>,
    node: &ReferralNode<I>,
    scope_id: I,
    lanes: &[Option<ReferralLane<I>>; MAX_PATH_NODES],
) -> Result<()> {
    validate_node_network(network, node)?;
    let depth = usize::from(node.depth);
    for (index, maybe_lane) in lanes.iter().enumerate() {
        match *maybe_lane {
            Some(lane) if index < depth => {
                if lane.network_id != network.id {
                    return Err(ReferralError::NetworkMismatch);
                }
                if lane.scope_id != scope_id {
                    return Err(ReferralError::LaneScopeMismatch);
                }
                if node.path_node_ids[index] != Some(lane.node_id)
                    || node.path_owners[index] != Some(lane.owner)
                {
                    return Err(ReferralError::LanePathMismatch);
                }
            }
            None if index >= depth => {}
            _ => return Err(ReferralError::LanePathMismatch),
        }
    }
    for (left, maybe_left_lane) in lanes[..depth].iter().enumerate() {
        let left_id = maybe_left_lane
            .as_ref()
            .ok_or(ReferralError::LanePathMismatch)?
            .id;
        for maybe_right_lane in &lanes[(left + 1)..depth] {
            let right_id = maybe_right_lane
                .as_ref()
                .ok_or(ReferralError::LanePathMismatch)?
                .id;
            if left_id == right_id {
                return Err(ReferralError::LanePathMismatch);
            }
        }
    }
    Ok(())
}

fn validate_active_offer<I: Eq>(
    network: &ReferralNetwork<I>,
    parent_node: &ReferralNode<I>,
    offer: &ReferralOffer<I>,
) -> Result<()> {
    validate_node_network(network, parent_node)?;
    if offer.network_id != network.id {
        return Err(ReferralError::NetworkMismatch);
    }
    if offer.parent_node_id != parent_node.id {
        return Err(ReferralError::OfferParentMismatch);
    }
    if !offer.open {
        return Err(ReferralError::OfferClosed);
    }
    match parent_node.active_offer_id.as_ref() {
        Some(active_id) if *active_id == offer.id => {}
        Some(_) => return Err(ReferralError::OfferVersionMismatch),
        None => return Err(ReferralError::NoActiveOffer),
    }
    if offer.version != parent_node.latest_offer_version {
        return Err(ReferralError::OfferVersionMismatch);
    }
    Ok(())
}
