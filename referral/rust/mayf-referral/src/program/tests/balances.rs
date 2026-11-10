use alloc::vec;

use super::*;
use crate::program::{SetBalanceArgs, SettleNodeInLaneArgs, set_balance, settle_node_in_lane};
use crate::{Change, Error};

#[test]
fn balance_change_earns_at_the_old_balance_before_replacing_it() {
    let receipt = set_balance(SetBalanceArgs {
        protocol: protocol(1),
        lane: lane(0, 100, 200, 2),
        member: member(1, None, 1),
        lineage: vec![],
        lane_nodes: vec![(1, position(100, 0, 2))],
        balance: 200,
        total_earned: number(250),
    })
    .unwrap();

    // The sole holder owns all 50 newly supplied credits, before its balance doubles.
    assert_eq!(receipt.member_next.kept, number(50));
    assert_eq!(receipt.balance_change, Change::Add(100));
    assert_eq!(receipt.lane_next.pool.balance, 200);
    assert_eq!(receipt.lane_next.pool.reward_index, fraction(5, 2));
    assert_eq!(receipt.lane_next.total_earned_seen, number(250));
    let position = position_for(&receipt.lane_nodes_next, 1);
    assert_eq!(position.balance, 200);
    assert_eq!(position.reward_index_seen, fraction(5, 2));
}

#[test]
fn first_balance_starts_after_old_holders_have_earned() {
    let receipt = set_balance(SetBalanceArgs {
        protocol: protocol(1),
        lane: lane(0, 100, 200, 2),
        member: member(2, Some(1), 0),
        lineage: vec![member(1, None, 1)],
        lane_nodes: vec![(1, position(100, 0, 2))],
        balance: 100,
        total_earned: number(250),
    })
    .unwrap();

    assert_eq!(receipt.member_next.kept, Number::ZERO);
    assert_eq!(receipt.lineage_next[0].kept, number(50));
    assert_eq!(receipt.member_next.lanes.bits(), 1);
    assert_eq!(receipt.lane_next.pool.balance, 200);
    let newcomer = position_for(&receipt.lane_nodes_next, 2);
    assert_eq!(newcomer.balance, 100);
    assert_eq!(newcomer.reward_index_seen, fraction(5, 2));
    let ancestor = position_for(&receipt.lane_nodes_next, 1);
    assert_eq!(ancestor.balance, 100);
    assert_eq!(ancestor.downline, 100);
    assert_eq!(ancestor.reward_index_seen, fraction(5, 2));
}

#[test]
fn balance_change_requires_an_ancestors_previously_materialized_position() {
    let result = set_balance(SetBalanceArgs {
        protocol: protocol(1),
        lane: lane(0, 0, 200, 2),
        member: member(2, Some(1), 0),
        // The ancestor withdrew, but its historical lane mask still requires its record.
        lineage: vec![member(1, None, 1)],
        lane_nodes: vec![],
        balance: 100,
        total_earned: number(250),
    });

    assert_eq!(result.err(), Some(Error::MissingLaneNode));
}

#[test]
fn balance_change_commits_a_projected_lane_without_recrediting_old_earnings() {
    let receipt = set_balance(SetBalanceArgs {
        protocol: protocol(1),
        // The node already consumed 7.5 through a read-only projection; the
        // persisted lane still has its old index. Four more credits arrive.
        lane: lane(0, 4, 0, 0),
        member: Member {
            kept: fraction(15, 2),
            ..member(1, None, 1)
        },
        lineage: vec![],
        lane_nodes: vec![(
            1,
            LaneNode {
                reward_index_seen: fraction(15, 8),
                ..position(4, 0, 0)
            },
        )],
        balance: 8,
        total_earned: fraction(23, 2),
    })
    .unwrap();

    assert_eq!(receipt.member_next.kept, fraction(23, 2));
    assert_eq!(receipt.lane_next.total_earned_seen, fraction(23, 2));
    assert_eq!(receipt.lane_next.pool.reward_index, fraction(23, 8));
    assert_eq!(receipt.lane_next.pool.balance, 8);
    assert_eq!(
        position_for(&receipt.lane_nodes_next, 1).reward_index_seen,
        fraction(23, 8)
    );
}

#[test]
fn node_settlement_accepts_fractional_lane_earnings() {
    let receipt = settle_node_in_lane(SettleNodeInLaneArgs {
        protocol: protocol(1),
        lane: lane(0, 4, 0, 0),
        node: member(1, None, 1),
        lineage: vec![],
        lane_nodes: vec![(1, position(4, 0, 0))],
        total_earned: fraction(15, 2),
    })
    .unwrap();

    assert_eq!(receipt.node_next.kept, fraction(15, 2));
    assert_eq!(
        position_for(&receipt.lane_nodes_next, 1).reward_index_seen,
        fraction(15, 8)
    );
}

#[test]
fn repeating_a_projected_lane_total_does_not_credit_a_node_twice() {
    let node = Member {
        kept: fraction(15, 2),
        ..member(1, None, 1)
    };
    let lane_nodes = vec![(
        1,
        LaneNode {
            reward_index_seen: fraction(15, 8),
            ..position(4, 0, 0)
        },
    )];
    let receipt = settle_node_in_lane(SettleNodeInLaneArgs {
        protocol: protocol(1),
        // Read-only node settlement need not have committed this lane's projection.
        lane: lane(0, 4, 0, 0),
        node: node.clone(),
        lineage: vec![],
        lane_nodes: lane_nodes.clone(),
        total_earned: fraction(15, 2),
    })
    .unwrap();

    assert_eq!(receipt.node_next, node);
    assert_eq!(receipt.lane_nodes_next, lane_nodes);
}
