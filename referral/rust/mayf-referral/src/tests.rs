use super::*;

type Id = u64;

/// Nodes on the root-to-leaf chain of the depth-2 test network.
const CHAIN: usize = 4;

#[derive(Clone, Copy)]
struct PathFixture {
    network: ReferralNetwork<Id>,
    node: ReferralNode<Id>,
}

fn network() -> ReferralNetwork<Id> {
    create_network(CreateNetworkArgs {
        max_depth: 2,
        override_numerator: 1,
        override_denominator: 10,
        network_id: 1,
        authority: 2,
        node_id: 10,
        owner: 110,
    })
    .expect("valid network")
    .network
}

fn publish_and_join(
    network: ReferralNetwork<Id>,
    parent: ReferralNode<Id>,
    node_id: Id,
) -> ReferralNode<Id> {
    let published = publish_offer(PublishOfferArgs {
        network,
        parent_node: parent,
        offer_id: 300 + node_id,
        join_fee: 7,
        join_fee_increment: 0,
    })
    .expect("parent can publish");
    let joined = join(&JoinArgs {
        max_join_fee: u64::MAX,
        network,
        parent_node: published.next_parent_node,
        offer: published.offer,
        node_id,
        owner: 100 + node_id,
    })
    .expect("child can join");
    assert_eq!(joined.join_fee_transfer.amount, 7);
    assert_eq!(joined.join_fee_transfer.recipient, parent.owner());
    joined.node
}

fn four_node_path() -> PathFixture {
    let created_root = create_network(CreateNetworkArgs {
        max_depth: 2,
        override_numerator: 1,
        override_denominator: 10,
        network_id: (network()).id(),
        authority: (network()).authority(),
        node_id: 10,
        owner: 110,
    })
    .expect("root can be created");

    let level_two = publish_and_join(created_root.network, created_root.node, 11);
    let level_three = publish_and_join(created_root.network, level_two, 12);
    let level_four = publish_and_join(created_root.network, level_three, 13);
    PathFixture {
        network: created_root.network,
        node: level_four,
    }
}

fn empty_lane_path(
    node: ReferralNode<Id>,
    scope_id: Id,
) -> [Option<ReferralLane<Id>>; MAX_PATH_NODES] {
    let mut lanes = [None; MAX_PATH_NODES];
    let node_ids = node.path_node_ids();
    let owners = node.path_owners();
    for index in 0..usize::from(node.depth()) {
        lanes[index] = Some(
            ReferralLane::try_from(ReferralLaneSnapshot {
                id: 700 + index as u64,
                network_id: node.network_id(),
                node_id: node_ids[index].expect("populated path node"),
                scope_id,
                owner: owners[index].expect("populated path owner"),
                direct_balance: 0,
                subtree_balance: 0,
                lifetime_deposited: 0,
                lifetime_withdrawn: 0,
            })
            .expect("empty lane is valid"),
        );
    }
    lanes
}

fn chain_nodes() -> (ReferralNetwork<Id>, [ReferralNode<Id>; CHAIN]) {
    let created_root = create_network(CreateNetworkArgs {
        max_depth: 2,
        override_numerator: 1,
        override_denominator: 10,
        network_id: (network()).id(),
        authority: (network()).authority(),
        node_id: 10,
        owner: 110,
    })
    .expect("root");
    let level_two = publish_and_join(created_root.network, created_root.node, 11);
    let level_three = publish_and_join(created_root.network, level_two, 12);
    let level_four = publish_and_join(created_root.network, level_three, 13);
    (
        created_root.network,
        [created_root.node, level_two, level_three, level_four],
    )
}

fn lane_path_from_state(
    lanes: &[ReferralLane<Id>; CHAIN],
    source_index: usize,
) -> [Option<ReferralLane<Id>>; MAX_PATH_NODES] {
    let mut path = [None; MAX_PATH_NODES];
    for (path_index, lane) in lanes[..=source_index].iter().rev().enumerate() {
        path[path_index] = Some(*lane);
    }
    path
}

fn apply_lane_path_to_state(
    lanes: &mut [ReferralLane<Id>; CHAIN],
    source_index: usize,
    path: [Option<ReferralLane<Id>>; MAX_PATH_NODES],
) {
    for (path_index, lane) in path[..=source_index].iter().enumerate() {
        lanes[source_index - path_index] = lane.expect("populated lane path");
    }
}

fn path_of(lanes: &[Option<ReferralLane<Id>>]) -> [Option<ReferralLane<Id>>; MAX_PATH_NODES] {
    let mut path = [None; MAX_PATH_NODES];
    path[..lanes.len()].copy_from_slice(lanes);
    path
}

fn deepest_chain() -> (ReferralNetwork<Id>, [ReferralNode<Id>; MAX_PATH_NODES]) {
    let created = create_network(CreateNetworkArgs {
        max_depth: (MAX_PATH_NODES - 2) as u8,
        override_numerator: 1,
        override_denominator: 10,
        network_id: 1,
        authority: 2,
        node_id: 10,
        owner: 110,
    })
    .expect("deepest supported network");
    let mut nodes = [created.node; MAX_PATH_NODES];
    for index in 1..MAX_PATH_NODES {
        nodes[index] = publish_and_join(created.network, nodes[index - 1], 10 + index as Id);
    }
    (created.network, nodes)
}

fn next_deterministic_u64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn offers_are_single_active_versioned_and_depth_capped() {
    let created_root = create_network(CreateNetworkArgs {
        max_depth: 2,
        override_numerator: 1,
        override_denominator: 10,
        network_id: (network()).id(),
        authority: (network()).authority(),
        node_id: 10,
        owner: 110,
    })
    .expect("root");
    let published = publish_offer(PublishOfferArgs {
        network: created_root.network,
        parent_node: created_root.node,
        offer_id: 300,
        join_fee: 7,
        join_fee_increment: 0,
    })
    .expect("publish");
    assert_eq!(published.offer.version(), 1);
    assert_eq!(
        publish_offer(PublishOfferArgs {
            network: created_root.network,
            parent_node: published.next_parent_node,
            offer_id: 301,
            join_fee: 8,
            join_fee_increment: 0,
        }),
        Err(ReferralError::ActiveOfferExists)
    );

    let replaced = replace_offer(ReplaceOfferArgs {
        network: created_root.network,
        parent_node: published.next_parent_node,
        active_offer: published.offer,
        replacement_offer_id: 301,
        join_fee: 9,
        join_fee_increment: 0,
    })
    .expect("replace");
    assert!(!replaced.closed_offer.is_open());
    assert_eq!(replaced.replacement_offer.version(), 2);
    assert_eq!(replaced.next_parent_node.active_offer_id(), Some(301));
    assert_eq!(
        join(&JoinArgs {
            max_join_fee: u64::MAX,
            network: created_root.network,
            parent_node: replaced.next_parent_node,
            offer: replaced.closed_offer,
            node_id: 11,
            owner: 111,
        }),
        Err(ReferralError::OfferClosed)
    );

    let closed = close_offer(CloseOfferArgs {
        network: created_root.network,
        parent_node: replaced.next_parent_node,
        offer: replaced.replacement_offer,
    })
    .expect("close");
    assert_eq!(closed.next_parent_node.active_offer_id(), None);
    assert!(!closed.closed_offer.is_open());

    let deepest = four_node_path();
    assert_eq!(
        publish_offer(PublishOfferArgs {
            network: deepest.network,
            parent_node: deepest.node,
            offer_id: 999,
            join_fee: 1,
            join_fee_increment: 0,
        }),
        Err(ReferralError::MaximumDepth)
    );
}

#[test]
fn every_path_must_end_at_the_networks_single_root() {
    let fixture = four_node_path();
    let mut snapshot: ReferralNetworkSnapshot<_> = fixture.network.into();
    snapshot.root_owner = 999;
    assert_eq!(
        validate_node_network(&snapshot.try_into().unwrap(), &fixture.node),
        Err(ReferralError::RootNodeMismatch)
    );
}

#[test]
fn lane_deposit_and_withdraw_update_one_direct_balance_and_every_subtree() {
    let fixture = four_node_path();
    let scope_id = 600;
    let lanes = empty_lane_path(fixture.node, scope_id);

    let deposited = deposit_into_lane(DepositIntoLaneArgs {
        network: fixture.network,
        node: fixture.node,
        scope_id,
        signer: fixture.node.owner(),
        lanes,
        amount: 40,
    })
    .expect("owner can deposit");

    assert!(deposited.next_lanes[CHAIN..].iter().all(Option::is_none));
    for (index, lane) in deposited.next_lanes[..CHAIN].iter().enumerate() {
        let lane = lane.expect("populated path lane");
        assert_eq!(lane.subtree_balance(), 40);
        if index == 0 {
            assert_eq!(lane.direct_balance(), 40);
            assert_eq!(lane.lifetime_deposited(), 40);
        } else {
            assert_eq!(lane.direct_balance(), 0);
            assert_eq!(lane.lifetime_deposited(), 0);
        }
        assert_eq!(lane.lifetime_withdrawn(), 0);
    }

    let withdrawn = withdraw_from_lane(WithdrawFromLaneArgs {
        network: fixture.network,
        node: fixture.node,
        scope_id,
        signer: fixture.node.owner(),
        lanes: deposited.next_lanes,
        amount: 15,
    })
    .expect("owner can withdraw direct units");

    assert!(withdrawn.next_lanes[CHAIN..].iter().all(Option::is_none));
    for (index, lane) in withdrawn.next_lanes[..CHAIN].iter().enumerate() {
        let lane = lane.expect("populated path lane");
        assert_eq!(lane.subtree_balance(), 25);
        if index == 0 {
            assert_eq!(lane.direct_balance(), 25);
            assert_eq!(lane.lifetime_deposited(), 40);
            assert_eq!(lane.lifetime_withdrawn(), 15);
            let snapshot: ReferralLaneSnapshot<_> = lane.into();
            assert_eq!(ReferralLane::try_from(snapshot), Ok(lane));
        } else {
            assert_eq!(lane.direct_balance(), 0);
            assert_eq!(lane.lifetime_deposited(), 0);
            assert_eq!(lane.lifetime_withdrawn(), 0);
        }
    }
}

#[test]
fn lane_actions_are_owner_gated_positive_and_limited_to_direct_units() {
    let fixture = four_node_path();
    let scope_id = 600;
    let lanes = empty_lane_path(fixture.node, scope_id);

    assert_eq!(
        deposit_into_lane(DepositIntoLaneArgs {
            network: fixture.network,
            node: fixture.node,
            scope_id,
            signer: 999,
            lanes,
            amount: 1,
        }),
        Err(ReferralError::LaneOwnerMismatch)
    );
    assert_eq!(
        withdraw_from_lane(WithdrawFromLaneArgs {
            network: fixture.network,
            node: fixture.node,
            scope_id,
            signer: 999,
            lanes,
            amount: 1,
        }),
        Err(ReferralError::LaneOwnerMismatch)
    );
    assert_eq!(
        deposit_into_lane(DepositIntoLaneArgs {
            network: fixture.network,
            node: fixture.node,
            scope_id,
            signer: fixture.node.owner(),
            lanes,
            amount: 0,
        }),
        Err(ReferralError::LaneDepositTooSmall)
    );
    assert_eq!(
        withdraw_from_lane(WithdrawFromLaneArgs {
            network: fixture.network,
            node: fixture.node,
            scope_id,
            signer: fixture.node.owner(),
            lanes,
            amount: 0,
        }),
        Err(ReferralError::LaneWithdrawalTooSmall)
    );
    assert_eq!(
        withdraw_from_lane(WithdrawFromLaneArgs {
            network: fixture.network,
            node: fixture.node,
            scope_id,
            signer: fixture.node.owner(),
            lanes,
            amount: 1,
        }),
        Err(ReferralError::LaneWithdrawalExceedsDirectBalance)
    );
}

#[test]
fn lane_actions_reject_wrong_scope_network_path_and_duplicate_accounts() {
    let fixture = four_node_path();
    let scope_id = 600;
    let lanes = empty_lane_path(fixture.node, scope_id);

    let assert_deposit_error = |lanes, expected| {
        assert_eq!(
            deposit_into_lane(DepositIntoLaneArgs {
                network: fixture.network,
                node: fixture.node,
                scope_id,
                signer: fixture.node.owner(),
                lanes,
                amount: 1,
            }),
            Err(expected)
        );
    };

    let mut missing = lanes;
    missing[2] = None;
    assert_deposit_error(missing, ReferralError::LanePathMismatch);

    let mut wrong_scope = lanes;
    let mut snapshot: ReferralLaneSnapshot<_> = wrong_scope[2].expect("lane").into();
    snapshot.scope_id = 601;
    wrong_scope[2] = Some(ReferralLane::try_from(snapshot).expect("locally valid lane"));
    assert_deposit_error(wrong_scope, ReferralError::LaneScopeMismatch);

    let mut wrong_network = lanes;
    let mut snapshot: ReferralLaneSnapshot<_> = wrong_network[2].expect("lane").into();
    snapshot.network_id = 999;
    wrong_network[2] = Some(ReferralLane::try_from(snapshot).expect("locally valid lane"));
    assert_deposit_error(wrong_network, ReferralError::NetworkMismatch);

    let mut wrong_node = lanes;
    let mut snapshot: ReferralLaneSnapshot<_> = wrong_node[2].expect("lane").into();
    snapshot.node_id = 999;
    wrong_node[2] = Some(ReferralLane::try_from(snapshot).expect("locally valid lane"));
    assert_deposit_error(wrong_node, ReferralError::LanePathMismatch);

    let mut wrong_owner = lanes;
    let mut snapshot: ReferralLaneSnapshot<_> = wrong_owner[2].expect("lane").into();
    snapshot.owner = 999;
    wrong_owner[2] = Some(ReferralLane::try_from(snapshot).expect("locally valid lane"));
    assert_deposit_error(wrong_owner, ReferralError::LanePathMismatch);

    let mut duplicate = lanes;
    let duplicate_id = duplicate[0].expect("lane").id();
    let mut snapshot: ReferralLaneSnapshot<_> = duplicate[2].expect("lane").into();
    snapshot.id = duplicate_id;
    duplicate[2] = Some(ReferralLane::try_from(snapshot).expect("locally valid lane"));
    assert_deposit_error(duplicate, ReferralError::LanePathMismatch);
}

#[test]
fn lane_snapshots_and_transitions_reject_impossible_accounting_and_overflow() {
    assert_eq!(
        ReferralLane::try_from(ReferralLaneSnapshot {
            id: 1,
            network_id: 2,
            node_id: 3,
            scope_id: 4,
            owner: 5,
            direct_balance: 2,
            subtree_balance: 1,
            lifetime_deposited: 2,
            lifetime_withdrawn: 0,
        }),
        Err(ReferralError::InvalidLaneAccounting)
    );
    assert_eq!(
        ReferralLane::try_from(ReferralLaneSnapshot {
            id: 1,
            network_id: 2,
            node_id: 3,
            scope_id: 4,
            owner: 5,
            direct_balance: 1,
            subtree_balance: 1,
            lifetime_deposited: 2,
            lifetime_withdrawn: 0,
        }),
        Err(ReferralError::InvalidLaneAccounting)
    );

    let fixture = four_node_path();
    let scope_id = 600;
    let mut lanes = empty_lane_path(fixture.node, scope_id);
    let mut source_snapshot: ReferralLaneSnapshot<_> = lanes[0].expect("source lane").into();
    source_snapshot.direct_balance = u64::MAX;
    source_snapshot.subtree_balance = u64::MAX;
    source_snapshot.lifetime_deposited = u128::from(u64::MAX);
    lanes[0] = Some(ReferralLane::try_from(source_snapshot).expect("valid full lane"));
    assert_eq!(
        deposit_into_lane(DepositIntoLaneArgs {
            network: fixture.network,
            node: fixture.node,
            scope_id,
            signer: fixture.node.owner(),
            lanes,
            amount: 1,
        }),
        Err(ReferralError::ArithmeticFailure)
    );

    let mut inconsistent_path = empty_lane_path(fixture.node, scope_id);
    let mut source_snapshot: ReferralLaneSnapshot<_> =
        inconsistent_path[0].expect("source lane").into();
    source_snapshot.direct_balance = 5;
    source_snapshot.subtree_balance = 5;
    source_snapshot.lifetime_deposited = 5;
    inconsistent_path[0] =
        Some(ReferralLane::try_from(source_snapshot).expect("locally valid source"));
    assert_eq!(
        withdraw_from_lane(WithdrawFromLaneArgs {
            network: fixture.network,
            node: fixture.node,
            scope_id,
            signer: fixture.node.owner(),
            lanes: inconsistent_path,
            amount: 5,
        }),
        Err(ReferralError::InvalidLaneAccounting)
    );
}

#[test]
fn create_lane_copies_node_identity_and_scopes_remain_independent() {
    let created_root = create_network(CreateNetworkArgs {
        max_depth: 2,
        override_numerator: 1,
        override_denominator: 10,
        network_id: (network()).id(),
        authority: (network()).authority(),
        node_id: 10,
        owner: 110,
    })
    .expect("root");
    let lane = create_lane(CreateLaneArgs {
        network: created_root.network,
        node: created_root.node,
        lane_id: 700,
        scope_id: 600,
    })
    .expect("lane")
    .lane;
    assert_eq!(lane.network_id(), created_root.network.id());
    assert_eq!(lane.node_id(), created_root.node.id());
    assert_eq!(lane.owner(), created_root.node.owner());
    assert_eq!(lane.scope_id(), 600);
    assert_eq!(lane.direct_balance(), 0);
    assert_eq!(lane.subtree_balance(), 0);

    let other_scope = create_lane(CreateLaneArgs {
        network: created_root.network,
        node: created_root.node,
        lane_id: 701,
        scope_id: 601,
    })
    .expect("other lane")
    .lane;
    let deposited = deposit_into_lane(DepositIntoLaneArgs {
        network: created_root.network,
        node: created_root.node,
        scope_id: 600,
        signer: created_root.node.owner(),
        lanes: path_of(&[Some(lane)]),
        amount: 9,
    })
    .expect("scoped deposit");
    assert_eq!(deposited.next_lanes[0].expect("lane").direct_balance(), 9);
    assert_eq!(other_scope.direct_balance(), 0);
    assert_eq!(other_scope.subtree_balance(), 0);

    assert_eq!(
        deposit_into_lane(DepositIntoLaneArgs {
            network: created_root.network,
            node: created_root.node,
            scope_id: 600,
            signer: created_root.node.owner(),
            lanes: path_of(&[Some(lane), Some(lane)]),
            amount: 1,
        }),
        Err(ReferralError::LanePathMismatch)
    );
}

#[test]
fn lane_subtree_totals_compose_across_siblings_and_direct_parent_units() {
    let created_root = create_network(CreateNetworkArgs {
        max_depth: 2,
        override_numerator: 1,
        override_denominator: 10,
        network_id: (network()).id(),
        authority: (network()).authority(),
        node_id: 10,
        owner: 110,
    })
    .expect("root");
    let published = publish_offer(PublishOfferArgs {
        network: created_root.network,
        parent_node: created_root.node,
        offer_id: 300,
        join_fee: 7,
        join_fee_increment: 0,
    })
    .expect("offer");
    let first = join(&JoinArgs {
        max_join_fee: u64::MAX,
        network: created_root.network,
        parent_node: published.next_parent_node,
        offer: published.offer,
        node_id: 11,
        owner: 111,
    })
    .expect("first child");
    let second = join(&JoinArgs {
        max_join_fee: u64::MAX,
        network: created_root.network,
        parent_node: published.next_parent_node,
        offer: published.offer,
        node_id: 12,
        owner: 112,
    })
    .expect("second child");
    let scope_id = 600;
    let root_lane = create_lane(CreateLaneArgs {
        network: created_root.network,
        node: created_root.node,
        lane_id: 700,
        scope_id,
    })
    .expect("root lane")
    .lane;
    let first_lane = create_lane(CreateLaneArgs {
        network: created_root.network,
        node: first.node,
        lane_id: 701,
        scope_id,
    })
    .expect("first lane")
    .lane;
    let second_lane = create_lane(CreateLaneArgs {
        network: created_root.network,
        node: second.node,
        lane_id: 702,
        scope_id,
    })
    .expect("second lane")
    .lane;

    let first_deposit = deposit_into_lane(DepositIntoLaneArgs {
        network: created_root.network,
        node: first.node,
        scope_id,
        signer: first.node.owner(),
        lanes: path_of(&[Some(first_lane), Some(root_lane)]),
        amount: 8,
    })
    .expect("first deposit");
    let root_after_first = first_deposit.next_lanes[1].expect("root lane");
    let second_deposit = deposit_into_lane(DepositIntoLaneArgs {
        network: created_root.network,
        node: second.node,
        scope_id,
        signer: second.node.owner(),
        lanes: path_of(&[Some(second_lane), Some(root_after_first)]),
        amount: 10,
    })
    .expect("second deposit");
    let root_after_children = second_deposit.next_lanes[1].expect("root lane");
    assert_eq!(root_after_children.direct_balance(), 0);
    assert_eq!(root_after_children.subtree_balance(), 18);
    assert_eq!(root_after_children.lifetime_deposited(), 0);

    let root_deposit = deposit_into_lane(DepositIntoLaneArgs {
        network: created_root.network,
        node: created_root.node,
        scope_id,
        signer: created_root.node.owner(),
        lanes: path_of(&[Some(root_after_children)]),
        amount: 4,
    })
    .expect("root direct deposit");
    let root_after_direct = root_deposit.next_lanes[0].expect("root lane");
    assert_eq!(root_after_direct.direct_balance(), 4);
    assert_eq!(root_after_direct.subtree_balance(), 22);
    assert_eq!(root_after_direct.lifetime_deposited(), 4);

    let first_withdrawal = withdraw_from_lane(WithdrawFromLaneArgs {
        network: created_root.network,
        node: first.node,
        scope_id,
        signer: first.node.owner(),
        lanes: path_of(&[first_deposit.next_lanes[0], Some(root_after_direct)]),
        amount: 3,
    })
    .expect("first withdrawal");
    assert_eq!(
        first_withdrawal.next_lanes[0]
            .expect("first lane")
            .direct_balance(),
        5
    );
    let final_root = first_withdrawal.next_lanes[1].expect("root lane");
    assert_eq!(final_root.direct_balance(), 4);
    assert_eq!(final_root.subtree_balance(), 19);
    assert_eq!(final_root.lifetime_deposited(), 4);
    assert_eq!(final_root.lifetime_withdrawn(), 0);
}

#[test]
fn deterministic_multi_scope_lane_operations_match_an_independent_oracle() {
    const SCOPE_COUNT: usize = 3;
    const OPERATION_COUNT: usize = 512;

    let (network, nodes) = chain_nodes();
    let scopes = [600, 601, 602];
    let mut lanes: [[ReferralLane<Id>; CHAIN]; SCOPE_COUNT] = core::array::from_fn(|scope_index| {
        core::array::from_fn(|node_index| {
            create_lane(CreateLaneArgs {
                network,
                node: nodes[node_index],
                lane_id: 10_000 + (scope_index as u64 * 100) + node_index as u64,
                scope_id: scopes[scope_index],
            })
            .expect("lane")
            .lane
        })
    });
    let mut expected_direct = [[0_u64; CHAIN]; SCOPE_COUNT];
    let mut expected_subtree = [[0_u64; CHAIN]; SCOPE_COUNT];
    let mut expected_deposited = [[0_u128; CHAIN]; SCOPE_COUNT];
    let mut expected_withdrawn = [[0_u128; CHAIN]; SCOPE_COUNT];
    let mut random_state = 0x4d59_5df4_d0f3_3173_u64;

    for operation in 0..OPERATION_COUNT {
        let random = next_deterministic_u64(&mut random_state);
        let scope_index = operation % SCOPE_COUNT;
        let source_index = (operation / SCOPE_COUNT) % CHAIN;
        let available = expected_direct[scope_index][source_index];
        let should_withdraw = random & 1 == 1 && available > 0;
        let amount = if should_withdraw {
            1 + (random.rotate_left(19) % available)
        } else {
            1 + (random.rotate_left(37) % 97)
        };
        let path = lane_path_from_state(&lanes[scope_index], source_index);

        let next_path = if should_withdraw {
            expected_direct[scope_index][source_index] -= amount;
            expected_withdrawn[scope_index][source_index] += u128::from(amount);
            for subtree in &mut expected_subtree[scope_index][..=source_index] {
                *subtree -= amount;
            }
            withdraw_from_lane(WithdrawFromLaneArgs {
                network,
                node: nodes[source_index],
                scope_id: scopes[scope_index],
                signer: nodes[source_index].owner(),
                lanes: path,
                amount,
            })
            .expect("oracle-authorized withdrawal")
            .next_lanes
        } else {
            expected_direct[scope_index][source_index] += amount;
            expected_deposited[scope_index][source_index] += u128::from(amount);
            for subtree in &mut expected_subtree[scope_index][..=source_index] {
                *subtree += amount;
            }
            deposit_into_lane(DepositIntoLaneArgs {
                network,
                node: nodes[source_index],
                scope_id: scopes[scope_index],
                signer: nodes[source_index].owner(),
                lanes: path,
                amount,
            })
            .expect("oracle-authorized deposit")
            .next_lanes
        };
        apply_lane_path_to_state(&mut lanes[scope_index], source_index, next_path);

        for (checked_scope, scope_lanes) in lanes.iter().enumerate() {
            for (node_index, lane) in scope_lanes.iter().enumerate() {
                assert_eq!(lane.scope_id(), scopes[checked_scope]);
                assert_eq!(lane.node_id(), nodes[node_index].id());
                assert_eq!(
                    lane.direct_balance(),
                    expected_direct[checked_scope][node_index]
                );
                assert_eq!(
                    lane.subtree_balance(),
                    expected_subtree[checked_scope][node_index]
                );
                assert_eq!(
                    lane.lifetime_deposited(),
                    expected_deposited[checked_scope][node_index]
                );
                assert_eq!(
                    lane.lifetime_withdrawn(),
                    expected_withdrawn[checked_scope][node_index]
                );
            }
        }
    }
}

#[test]
fn an_intermediate_lane_composes_its_direct_and_downline_balances() {
    let (network, nodes) = chain_nodes();
    let scope_id = 600;
    let mut lanes: [ReferralLane<Id>; CHAIN] = core::array::from_fn(|index| {
        create_lane(CreateLaneArgs {
            network,
            node: nodes[index],
            lane_id: 700 + index as u64,
            scope_id,
        })
        .expect("lane")
        .lane
    });

    let middle_deposit = deposit_into_lane(DepositIntoLaneArgs {
        network,
        node: nodes[1],
        scope_id,
        signer: nodes[1].owner(),
        lanes: lane_path_from_state(&lanes, 1),
        amount: 10,
    })
    .expect("middle deposit");
    apply_lane_path_to_state(&mut lanes, 1, middle_deposit.next_lanes);

    let downline_deposit = deposit_into_lane(DepositIntoLaneArgs {
        network,
        node: nodes[2],
        scope_id,
        signer: nodes[2].owner(),
        lanes: lane_path_from_state(&lanes, 2),
        amount: 7,
    })
    .expect("downline deposit");
    apply_lane_path_to_state(&mut lanes, 2, downline_deposit.next_lanes);

    assert_eq!(lanes[0].direct_balance(), 0);
    assert_eq!(lanes[0].subtree_balance(), 17);
    assert_eq!(lanes[1].direct_balance(), 10);
    assert_eq!(lanes[1].subtree_balance(), 17);
    assert_eq!(lanes[1].lifetime_deposited(), 10);
    assert_eq!(lanes[2].direct_balance(), 7);
    assert_eq!(lanes[2].subtree_balance(), 7);

    let middle_withdrawal = withdraw_from_lane(WithdrawFromLaneArgs {
        network,
        node: nodes[1],
        scope_id,
        signer: nodes[1].owner(),
        lanes: lane_path_from_state(&lanes, 1),
        amount: 4,
    })
    .expect("middle withdrawal");
    apply_lane_path_to_state(&mut lanes, 1, middle_withdrawal.next_lanes);

    assert_eq!(lanes[0].subtree_balance(), 13);
    assert_eq!(lanes[1].direct_balance(), 6);
    assert_eq!(lanes[1].subtree_balance(), 13);
    assert_eq!(lanes[1].lifetime_deposited(), 10);
    assert_eq!(lanes[1].lifetime_withdrawn(), 4);
    assert_eq!(lanes[2].direct_balance(), 7);
    assert_eq!(lanes[2].subtree_balance(), 7);
}

#[test]
fn upline_subtree_overflow_and_other_failures_leave_inputs_unchanged() {
    let (network, nodes) = chain_nodes();
    let scope_id = 600;
    let child_lane = create_lane(CreateLaneArgs {
        network,
        node: nodes[1],
        lane_id: 701,
        scope_id,
    })
    .expect("child lane")
    .lane;
    let root_lane = ReferralLane::try_from(ReferralLaneSnapshot {
        id: 700,
        network_id: network.id(),
        node_id: nodes[0].id(),
        scope_id,
        owner: nodes[0].owner(),
        direct_balance: 0,
        subtree_balance: u64::MAX,
        lifetime_deposited: 0,
        lifetime_withdrawn: 0,
    })
    .expect("locally valid saturated upline lane");
    let lanes = path_of(&[Some(child_lane), Some(root_lane)]);
    let before = lanes;

    assert_eq!(
        deposit_into_lane(DepositIntoLaneArgs {
            network,
            node: nodes[1],
            scope_id,
            signer: nodes[1].owner(),
            lanes,
            amount: 1,
        }),
        Err(ReferralError::ArithmeticFailure)
    );
    assert_eq!(lanes, before);

    assert_eq!(
        deposit_into_lane(DepositIntoLaneArgs {
            network,
            node: nodes[1],
            scope_id,
            signer: 999,
            lanes,
            amount: 1,
        }),
        Err(ReferralError::LaneOwnerMismatch)
    );
    assert_eq!(lanes, before);

    assert_eq!(
        withdraw_from_lane(WithdrawFromLaneArgs {
            network,
            node: nodes[1],
            scope_id,
            signer: nodes[1].owner(),
            lanes,
            amount: 1,
        }),
        Err(ReferralError::LaneWithdrawalExceedsDirectBalance)
    );
    assert_eq!(lanes, before);
}

#[test]
fn offer_lifecycle_supports_reopening_after_close_and_enforces_active_state() {
    let created_root = create_network(CreateNetworkArgs {
        max_depth: 2,
        override_numerator: 1,
        override_denominator: 10,
        network_id: (network()).id(),
        authority: (network()).authority(),
        node_id: 10,
        owner: 110,
    })
    .expect("root");

    let published_v1 = publish_offer(PublishOfferArgs {
        network: created_root.network,
        parent_node: created_root.node,
        offer_id: 300,
        join_fee: 7,
        join_fee_increment: 0,
    })
    .expect("published v1");
    assert_eq!(published_v1.offer.version(), 1);
    assert!(published_v1.offer.is_open());
    assert_eq!(published_v1.next_parent_node.active_offer_id(), Some(300));

    let closed_v1 = close_offer(CloseOfferArgs {
        network: created_root.network,
        parent_node: published_v1.next_parent_node,
        offer: published_v1.offer,
    })
    .expect("closed v1");
    assert_eq!(closed_v1.next_parent_node.active_offer_id(), None);
    assert_eq!(closed_v1.next_parent_node.latest_offer_version(), 1);
    assert!(!closed_v1.closed_offer.is_open());

    assert_eq!(
        close_offer(CloseOfferArgs {
            network: created_root.network,
            parent_node: closed_v1.next_parent_node,
            offer: closed_v1.closed_offer,
        }),
        Err(ReferralError::OfferClosed)
    );

    assert_eq!(
        replace_offer(ReplaceOfferArgs {
            network: created_root.network,
            parent_node: closed_v1.next_parent_node,
            active_offer: closed_v1.closed_offer,
            replacement_offer_id: 301,
            join_fee: 10,
            join_fee_increment: 0,
        }),
        Err(ReferralError::OfferClosed)
    );

    let published_v2 = publish_offer(PublishOfferArgs {
        network: created_root.network,
        parent_node: closed_v1.next_parent_node,
        offer_id: 301,
        join_fee: 15,
        join_fee_increment: 0,
    })
    .expect("published v2 after closing v1");
    assert_eq!(published_v2.offer.version(), 2);
    assert_eq!(published_v2.next_parent_node.active_offer_id(), Some(301));
    assert_eq!(published_v2.next_parent_node.latest_offer_version(), 2);

    assert_eq!(
        replace_offer(ReplaceOfferArgs {
            network: created_root.network,
            parent_node: published_v2.next_parent_node,
            active_offer: published_v2.offer,
            replacement_offer_id: 301,
            join_fee: 20,
            join_fee_increment: 0,
        }),
        Err(ReferralError::OfferIdReused)
    );

    assert_eq!(
        publish_offer(PublishOfferArgs {
            network: created_root.network,
            parent_node: published_v2.next_parent_node,
            offer_id: 302,
            join_fee: 20,
            join_fee_increment: 0,
        }),
        Err(ReferralError::ActiveOfferExists)
    );

    let joined = join(&JoinArgs {
        max_join_fee: u64::MAX,
        network: created_root.network,
        parent_node: published_v2.next_parent_node,
        offer: published_v2.offer,
        node_id: 11,
        owner: 111,
    })
    .expect("join with v2");
    assert_eq!(joined.node.path_offer_versions()[0], 2);
    assert_eq!(joined.node.path_offer_ids()[0], Some(301));

    assert_eq!(
        join(&JoinArgs {
            max_join_fee: u64::MAX,
            network: created_root.network,
            parent_node: published_v2.next_parent_node,
            offer: closed_v1.closed_offer,
            node_id: 12,
            owner: 112,
        }),
        Err(ReferralError::OfferClosed)
    );
}

#[test]
fn join_enforces_upline_uniqueness_depth_caps_and_root_integrity() {
    let fixture = four_node_path();

    assert_eq!(
        join(&JoinArgs {
            max_join_fee: u64::MAX,
            network: fixture.network,

            parent_node: fixture.node,
            offer: ReferralOffer::try_from(ReferralOfferSnapshot {
                id: 999,
                network_id: fixture.network.id(),
                parent_node_id: fixture.node.id(),
                version: 1,
                open: true,
                join_fee: 0,
                join_fee_increment: 0,
            })
            .unwrap(),
            node_id: 99,
            owner: 999,
        }),
        Err(ReferralError::MaximumDepth)
    );

    let created_root = create_network(CreateNetworkArgs {
        max_depth: 2,
        override_numerator: 1,
        override_denominator: 10,
        network_id: (network()).id(),
        authority: (network()).authority(),
        node_id: 10,
        owner: 110,
    })
    .expect("root");
    let network = created_root.network;
    let root_node = created_root.node;
    let level_two = publish_and_join(network, root_node, 11);
    let level_three = publish_and_join(network, level_two, 12);
    assert_eq!(level_three.depth(), 3);

    let published = publish_offer(PublishOfferArgs {
        network,
        parent_node: level_three,
        offer_id: 400,
        join_fee: 0,
        join_fee_increment: 0,
    })
    .expect("published on depth 3");

    assert_eq!(
        join(&JoinArgs {
            max_join_fee: u64::MAX,
            network,
            parent_node: published.next_parent_node,
            offer: published.offer,
            node_id: level_three.id(),
            owner: 555,
        }),
        Err(ReferralError::DuplicatePathNode)
    );

    assert_eq!(
        join(&JoinArgs {
            max_join_fee: u64::MAX,
            network,
            parent_node: published.next_parent_node,
            offer: published.offer,
            node_id: level_two.id(),
            owner: 555,
        }),
        Err(ReferralError::DuplicatePathNode)
    );

    assert_eq!(
        join(&JoinArgs {
            max_join_fee: u64::MAX,
            network,
            parent_node: published.next_parent_node,
            offer: published.offer,
            node_id: root_node.id(),
            owner: 555,
        }),
        Err(ReferralError::DuplicatePathNode)
    );

    assert!(
        join(&JoinArgs {
            max_join_fee: u64::MAX,
            network,
            parent_node: published.next_parent_node,
            offer: published.offer,
            node_id: 13,
            owner: level_three.owner(),
        })
        .is_ok()
    );

    assert!(
        join(&JoinArgs {
            max_join_fee: u64::MAX,
            network,
            parent_node: published.next_parent_node,
            offer: published.offer,
            node_id: 13,
            owner: level_two.owner(),
        })
        .is_ok()
    );

    assert!(
        join(&JoinArgs {
            max_join_fee: u64::MAX,
            network,
            parent_node: published.next_parent_node,
            offer: published.offer,
            node_id: 13,
            owner: root_node.owner(),
        })
        .is_ok()
    );

    let foreign_network = create_network(CreateNetworkArgs {
        max_depth: 2,
        override_numerator: 1,
        override_denominator: 10,
        network_id: 99,
        authority: 2,
        node_id: 10,
        owner: 110,
    })
    .expect("foreign")
    .network;
    let mut wrong_terminus: ReferralNodeSnapshot<_> = published.next_parent_node.into();
    wrong_terminus.root_id = 999;
    assert_eq!(
        join(&JoinArgs {
            max_join_fee: u64::MAX,
            network,
            parent_node: wrong_terminus.try_into().unwrap(),
            offer: published.offer,
            node_id: 13,
            owner: 113,
        }),
        Err(ReferralError::RootNodeMismatch)
    );

    let foreign_offer = publish_offer(PublishOfferArgs {
        network: foreign_network,
        parent_node: create_network(CreateNetworkArgs {
            max_depth: 2,
            override_numerator: 1,
            override_denominator: 10,
            network_id: (foreign_network).id(),
            authority: (foreign_network).authority(),
            node_id: 10,
            owner: 110,
        })
        .unwrap()
        .node,
        offer_id: 888,
        join_fee: 0,
        join_fee_increment: 0,
    })
    .unwrap()
    .offer;
    assert_eq!(
        join(&JoinArgs {
            max_join_fee: u64::MAX,
            network,
            parent_node: published.next_parent_node,
            offer: foreign_offer,
            node_id: 13,
            owner: 113,
        }),
        Err(ReferralError::NetworkMismatch)
    );

    let successful = join(&JoinArgs {
        max_join_fee: u64::MAX,
        network,
        parent_node: published.next_parent_node,
        offer: published.offer,
        node_id: 13,
        owner: 113,
    })
    .expect("successful zero fee join");
    assert_eq!(successful.join_fee_transfer.amount, 0);
    assert_eq!(successful.join_fee_transfer.payer, 113);
    assert_eq!(successful.join_fee_transfer.recipient, level_three.owner());
    assert_eq!(successful.node.depth(), 4);
}

#[test]
fn lane_multi_tier_tree_enforces_subtree_sum_and_downline_isolation() {
    let created_root = create_network(CreateNetworkArgs {
        max_depth: 2,
        override_numerator: 1,
        override_denominator: 10,
        network_id: (network()).id(),
        authority: (network()).authority(),
        node_id: 10,
        owner: 110,
    })
    .expect("root");
    let net = created_root.network;

    let root_offer = publish_offer(PublishOfferArgs {
        network: net,
        parent_node: created_root.node,
        offer_id: 300,
        join_fee: 1,
        join_fee_increment: 0,
    })
    .expect("root offer");

    let child_a = join(&JoinArgs {
        max_join_fee: u64::MAX,
        network: net,

        parent_node: root_offer.next_parent_node,
        offer: root_offer.offer,
        node_id: 11,
        owner: 111,
    })
    .expect("child a");

    let child_b = join(&JoinArgs {
        max_join_fee: u64::MAX,
        network: net,

        parent_node: root_offer.next_parent_node,
        offer: root_offer.offer,
        node_id: 12,
        owner: 112,
    })
    .expect("child b");

    let a_offer = publish_offer(PublishOfferArgs {
        network: net,
        parent_node: child_a.node,
        offer_id: 301,
        join_fee: 1,
        join_fee_increment: 0,
    })
    .expect("a offer");

    let grandchild_c = join(&JoinArgs {
        max_join_fee: u64::MAX,
        network: net,

        parent_node: a_offer.next_parent_node,
        offer: a_offer.offer,
        node_id: 13,
        owner: 113,
    })
    .expect("grandchild c");

    let scope = 888;
    let mut root_lane = create_lane(CreateLaneArgs {
        network: net,
        node: created_root.node,
        lane_id: 710,
        scope_id: scope,
    })
    .unwrap()
    .lane;

    let mut a_lane = create_lane(CreateLaneArgs {
        network: net,
        node: a_offer.next_parent_node,
        lane_id: 711,
        scope_id: scope,
    })
    .unwrap()
    .lane;

    let mut b_lane = create_lane(CreateLaneArgs {
        network: net,
        node: child_b.node,
        lane_id: 712,
        scope_id: scope,
    })
    .unwrap()
    .lane;

    let mut c_lane = create_lane(CreateLaneArgs {
        network: net,
        node: grandchild_c.node,
        lane_id: 713,
        scope_id: scope,
    })
    .unwrap()
    .lane;

    let dep_c = deposit_into_lane(DepositIntoLaneArgs {
        network: net,
        node: grandchild_c.node,
        scope_id: scope,
        signer: grandchild_c.node.owner(),
        lanes: path_of(&[Some(c_lane), Some(a_lane), Some(root_lane)]),
        amount: 50,
    })
    .unwrap();
    c_lane = dep_c.next_lanes[0].unwrap();
    a_lane = dep_c.next_lanes[1].unwrap();
    root_lane = dep_c.next_lanes[2].unwrap();

    let dep_a = deposit_into_lane(DepositIntoLaneArgs {
        network: net,
        node: a_offer.next_parent_node,
        scope_id: scope,
        signer: a_offer.next_parent_node.owner(),
        lanes: path_of(&[Some(a_lane), Some(root_lane)]),
        amount: 30,
    })
    .unwrap();
    a_lane = dep_a.next_lanes[0].unwrap();
    root_lane = dep_a.next_lanes[1].unwrap();

    let dep_b = deposit_into_lane(DepositIntoLaneArgs {
        network: net,
        node: child_b.node,
        scope_id: scope,
        signer: child_b.node.owner(),
        lanes: path_of(&[Some(b_lane), Some(root_lane)]),
        amount: 20,
    })
    .unwrap();
    b_lane = dep_b.next_lanes[0].unwrap();
    root_lane = dep_b.next_lanes[1].unwrap();

    let dep_root = deposit_into_lane(DepositIntoLaneArgs {
        network: net,
        node: created_root.node,
        scope_id: scope,
        signer: created_root.node.owner(),
        lanes: path_of(&[Some(root_lane)]),
        amount: 10,
    })
    .unwrap();
    root_lane = dep_root.next_lanes[0].unwrap();

    assert_eq!(c_lane.direct_balance(), 50);
    assert_eq!(c_lane.subtree_balance(), 50);
    assert_eq!(a_lane.direct_balance(), 30);
    assert_eq!(a_lane.subtree_balance(), 80);
    assert_eq!(b_lane.direct_balance(), 20);
    assert_eq!(b_lane.subtree_balance(), 20);
    assert_eq!(root_lane.direct_balance(), 10);
    assert_eq!(root_lane.subtree_balance(), 110);

    assert_eq!(
        root_lane.subtree_balance(),
        root_lane.direct_balance() + a_lane.subtree_balance() + b_lane.subtree_balance()
    );
    assert_eq!(
        a_lane.subtree_balance(),
        a_lane.direct_balance() + c_lane.subtree_balance()
    );

    assert_eq!(
        withdraw_from_lane(WithdrawFromLaneArgs {
            network: net,
            node: a_offer.next_parent_node,
            scope_id: scope,
            signer: a_offer.next_parent_node.owner(),
            lanes: path_of(&[Some(a_lane), Some(root_lane)]),
            amount: 31,
        }),
        Err(ReferralError::LaneWithdrawalExceedsDirectBalance)
    );

    let w_a = withdraw_from_lane(WithdrawFromLaneArgs {
        network: net,
        node: a_offer.next_parent_node,
        scope_id: scope,
        signer: a_offer.next_parent_node.owner(),
        lanes: path_of(&[Some(a_lane), Some(root_lane)]),
        amount: 30,
    })
    .unwrap();
    a_lane = w_a.next_lanes[0].unwrap();
    root_lane = w_a.next_lanes[1].unwrap();
    assert_eq!(a_lane.direct_balance(), 0);
    assert_eq!(a_lane.subtree_balance(), 50);
    assert_eq!(root_lane.subtree_balance(), 80);

    let w_c = withdraw_from_lane(WithdrawFromLaneArgs {
        network: net,
        node: grandchild_c.node,
        scope_id: scope,
        signer: grandchild_c.node.owner(),
        lanes: path_of(&[Some(c_lane), Some(a_lane), Some(root_lane)]),
        amount: 50,
    })
    .unwrap();
    c_lane = w_c.next_lanes[0].unwrap();
    a_lane = w_c.next_lanes[1].unwrap();
    root_lane = w_c.next_lanes[2].unwrap();
    assert_eq!(c_lane.direct_balance(), 0);
    assert_eq!(c_lane.subtree_balance(), 0);
    assert_eq!(a_lane.subtree_balance(), 0);
    assert_eq!(root_lane.subtree_balance(), 30);

    let w_b = withdraw_from_lane(WithdrawFromLaneArgs {
        network: net,
        node: child_b.node,
        scope_id: scope,
        signer: child_b.node.owner(),
        lanes: path_of(&[Some(b_lane), Some(root_lane)]),
        amount: 20,
    })
    .unwrap();
    b_lane = w_b.next_lanes[0].unwrap();
    root_lane = w_b.next_lanes[1].unwrap();
    assert_eq!(b_lane.direct_balance(), 0);
    assert_eq!(b_lane.subtree_balance(), 0);
    assert_eq!(root_lane.subtree_balance(), 10);

    let w_root = withdraw_from_lane(WithdrawFromLaneArgs {
        network: net,
        node: created_root.node,
        scope_id: scope,
        signer: created_root.node.owner(),
        lanes: path_of(&[Some(root_lane)]),
        amount: 10,
    })
    .unwrap();
    root_lane = w_root.next_lanes[0].unwrap();
    assert_eq!(root_lane.direct_balance(), 0);
    assert_eq!(root_lane.subtree_balance(), 0);

    assert_eq!(c_lane.lifetime_deposited(), 50);
    assert_eq!(c_lane.lifetime_withdrawn(), 50);
    assert_eq!(a_lane.lifetime_deposited(), 30);
    assert_eq!(a_lane.lifetime_withdrawn(), 30);
    assert_eq!(b_lane.lifetime_deposited(), 20);
    assert_eq!(b_lane.lifetime_withdrawn(), 20);
    assert_eq!(root_lane.lifetime_deposited(), 10);
    assert_eq!(root_lane.lifetime_withdrawn(), 10);
}

#[test]
fn network_policy_validates_depth_and_rounds_override_rate_toward_upline() {
    let snapshot: ReferralNetworkSnapshot<_> = network().into();
    let raw_tenth_ceil = (1_u128 << 64).div_ceil(10);
    assert_eq!(network().override_rate().to_scaled_u128(), raw_tenth_ceil);
    for (max_depth, numerator, denominator) in [(7, 1, 10), (2, 1, 2), (2, 1, 0), (0, 1, 1)] {
        let invalid = ReferralNetworkSnapshot {
            max_depth,
            override_numerator: numerator,
            override_denominator: denominator,
            ..snapshot
        };
        assert_eq!(
            ReferralNetwork::try_from(invalid),
            Err(ReferralError::InvalidConfiguration)
        );
    }
    let zero = ReferralNetwork::try_from(ReferralNetworkSnapshot {
        override_numerator: 0,
        ..snapshot
    })
    .expect("zero override rate is permitted");
    assert_eq!(zero.config().override_rate, ReferralPoints::ZERO);
    let deepest = ReferralNetwork::try_from(ReferralNetworkSnapshot {
        max_depth: 6,
        ..snapshot
    })
    .expect("the deepest supported network is valid");
    assert_eq!(deepest.max_depth(), 6);
}

#[test]
fn network_configs_widen_the_rounded_rate_exactly_and_survive_snapshot_round_trips() {
    let snapshot: ReferralNetworkSnapshot<_> = network().into();
    for (numerator, denominator) in [(0, 1), (1, u32::MAX), (1, 10), (u32::MAX - 1, u32::MAX)] {
        let snapshot = ReferralNetworkSnapshot {
            max_depth: 0,
            override_numerator: numerator,
            override_denominator: denominator,
            ..snapshot
        };
        let network = ReferralNetwork::try_from(snapshot).unwrap();
        let rate: ReferralRate = network.override_rate();
        let config: ReferralTreeConfig = network.config();
        let expected_raw = (u128::from(numerator) << 64).div_ceil(u128::from(denominator));
        assert_eq!(rate.to_scaled_u128(), expected_raw);
        assert_eq!(ReferralRate::try_from(config.override_rate), Ok(rate));
        assert_eq!(config.max_depth, snapshot.max_depth);

        let stored = ReferralNetworkSnapshot::from(network);
        assert_eq!(stored, snapshot);
        assert_eq!(ReferralNetwork::try_from(stored).unwrap().config(), config);
    }
}

#[test]
fn concrete_tree_settles_a_full_u64_scope_over_a_full_u64_lifetime() {
    let (network, nodes) = chain_nodes();
    let scope_id = 600;
    let mut lanes = nodes.map(|node| {
        create_lane(CreateLaneArgs {
            network,
            node,
            lane_id: 700 + node.id(),
            scope_id,
        })
        .unwrap()
        .lane
    });
    // The administrative root is outside the participating tree. Fund a top
    // member and its child through the concrete program's token transitions.
    for (source, amount) in [(1, u64::MAX - 1), (2, 1)] {
        let deposited = deposit_into_lane(DepositIntoLaneArgs {
            network,
            node: nodes[source],
            scope_id,
            signer: nodes[source].owner(),
            lanes: lane_path_from_state(&lanes, source),
            amount,
        })
        .unwrap();
        apply_lane_path_to_state(&mut lanes, source, deposited.next_lanes);
    }
    assert_eq!(lanes[0].subtree_balance(), u64::MAX);
    let path = [
        ReferralTreeNode {
            deposit: lanes[1].direct_balance(),
            downline_deposits: lanes[1].subtree_balance() - lanes[1].direct_balance(),
            child_deposits: lanes[2].direct_balance(),
            sharing: ReferralPoints::from(ReferralRate::ONE),
            ..ReferralTreeNode::default()
        },
        ReferralTreeNode {
            deposit: lanes[2].direct_balance(),
            ..ReferralTreeNode::default()
        },
    ];
    let settled = referral_tree::checkpoint_path(
        &path,
        referral_tree::CheckpointPathArgs {
            config: network.config(),
            time: u64::MAX,
        },
    )
    .unwrap()
    .next_path;
    // The top member shares everything with its one-unit child. Derive the
    // exact Q64 raw bytes using native integer arithmetic, independently of
    // the driver's multiplication. The result exceeds every u64-sized grid.
    let whole_point_time = u128::from(u64::MAX) * u128::from(u64::MAX);
    let mut expected = [0_u8; 32];
    expected[8..24].copy_from_slice(&whole_point_time.to_le_bytes());
    assert_eq!(settled[0].point_time, ReferralPoints::ZERO);
    assert_eq!(settled[1].point_time.to_le_bytes(), expected);
    assert_eq!(path[0].last_time, 0);
    assert_eq!(path[1].point_time, ReferralPoints::ZERO);
}

#[test]
fn the_deepest_network_admits_seven_levels_and_propagates_deposits() {
    let (network, nodes) = deepest_chain();
    let leaf = nodes[MAX_PATH_NODES - 1];
    assert_eq!(usize::from(leaf.depth()), MAX_PATH_NODES);
    assert_eq!(
        publish_offer(PublishOfferArgs {
            network,
            parent_node: leaf,
            offer_id: 999,
            join_fee: 0,
            join_fee_increment: 0,
        }),
        Err(ReferralError::MaximumDepth)
    );

    let scope_id = 600;
    let deposited = deposit_into_lane(DepositIntoLaneArgs {
        network,
        node: leaf,
        scope_id,
        signer: leaf.owner(),
        lanes: empty_lane_path(leaf, scope_id),
        amount: 25,
    })
    .expect("deposit through the full path");
    for lane in deposited.next_lanes.iter().flatten() {
        assert_eq!(lane.subtree_balance(), 25);
    }
    assert_eq!(
        deposited.next_lanes[0].expect("leaf lane").direct_balance(),
        25
    );
}

#[test]
fn admission_prices_progress_with_successful_joins_and_updates_are_partial() {
    let (network, nodes) = chain_nodes();
    let published = publish_offer(PublishOfferArgs {
        network,
        parent_node: nodes[0],
        offer_id: 991,
        join_fee: 7,
        join_fee_increment: 3,
    })
    .expect("publish");
    let args = JoinArgs {
        network,
        parent_node: published.next_parent_node,
        offer: published.offer,
        node_id: 99,
        owner: 199,
        max_join_fee: 6,
    };
    assert_eq!(join(&args), Err(ReferralError::JoinFeeExceeded));
    let joined = join(&JoinArgs {
        max_join_fee: 7,
        ..args
    })
    .expect("bounded admission");
    assert_eq!(
        joined.join_fee_transfer,
        FeeTransfer {
            payer: 199,
            recipient: nodes[0].owner(),
            amount: 7
        }
    );
    assert_eq!(joined.next_offer.join_fee(), 10);
    assert_eq!(
        joined.next_parent_node.joined_children(),
        nodes[0].joined_children() + 1
    );
    assert_eq!(published.offer.join_fee(), 7);

    let repriced = update_offer_pricing(UpdateOfferPricingArgs {
        network,
        parent_node: joined.next_parent_node,
        offer: joined.next_offer,
        join_fee: Some(20),
        join_fee_increment: None,
    })
    .expect("replace current price only");
    assert_eq!(
        (repriced.join_fee(), repriced.join_fee_increment()),
        (20, 3)
    );
    let overflowing = update_offer_pricing(UpdateOfferPricingArgs {
        network,
        parent_node: joined.next_parent_node,
        offer: repriced,
        join_fee: Some(u64::MAX),
        join_fee_increment: None,
    })
    .unwrap();
    assert_eq!(
        join(&JoinArgs {
            parent_node: joined.next_parent_node,
            offer: overflowing,
            max_join_fee: u64::MAX,
            ..args
        }),
        Err(ReferralError::ArithmeticFailure)
    );
}

#[test]
fn node_snapshots_reject_missing_edges_duplicate_nodes_and_unpopulated_path_gaps() {
    let original: ReferralNodeSnapshot<_> = four_node_path().node.into();
    let mut missing_offer = original;
    missing_offer.path_offer_ids[1] = None;
    let mut missing_version = original;
    missing_version.path_offer_versions[1] = 0;
    let mut missing_owner = original;
    missing_owner.path_owners[1] = None;
    let mut max_depth_offer: ReferralNodeSnapshot<_> = deepest_chain().1[MAX_PATH_NODES - 1].into();
    max_depth_offer.active_offer_id = Some(999);
    max_depth_offer.latest_offer_version = 1;
    for snapshot in [
        missing_offer,
        missing_version,
        missing_owner,
        max_depth_offer,
    ] {
        assert_eq!(
            ReferralNode::try_from(snapshot),
            Err(ReferralError::InvalidNodePath)
        );
    }
    let mut duplicate = original;
    duplicate.path_node_ids[1] = duplicate.path_node_ids[0];
    assert_eq!(
        ReferralNode::try_from(duplicate),
        Err(ReferralError::DuplicatePathNode)
    );
    assert_eq!(
        ReferralNodeSnapshot::from(ReferralNode::try_from(original).unwrap()),
        original
    );
}
