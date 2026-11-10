use alloc::vec;

use super::*;
use crate::program::{
    SetCreditStakeArgs, SetSharingArgs, SettleFamilyArgs, set_credit_stake, set_sharing,
    settle_family,
};
use crate::{Change, Error};

/// Two independent lane totals provide four and eight fresh credits. With a
/// 25% override and 50% sharing, the parent retains 1.5 and markets 1.5. Each
/// child owns half of the two-unit escrow, so each earns 0.75. Source ordering
/// deliberately differs from position ordering: lane IDs must determine pairing.
fn family() -> Family {
    Family {
        protocol: protocol(2),
        parent: Member {
            sharing: percent(50),
            child_escrow: 2,
            ..member(1, None, 3)
        },
        child: Member {
            escrow: 1,
            ..member(2, Some(1), 0)
        },
        lanes: vec![
            (lane(1, 4, 20, 3), number(28)),
            (lane(0, 4, 10, 2), number(14)),
        ],
        positions: vec![(id(0), position(0, 4, 2)), (id(1), position(0, 4, 3))],
    }
}

impl Family {
    fn sharing_args(self) -> SetSharingArgs<u64, Number> {
        SetSharingArgs {
            protocol: self.protocol,
            member: self.parent,
            lanes: self.lanes,
            lane_nodes: self.positions,
            sharing: MicroBasisPoints::ZERO,
        }
    }
}

#[test]
fn sharing_change_settles_each_lane_under_the_old_rate() {
    let receipt = set_sharing(family().sharing_args()).unwrap();

    assert_eq!(receipt.member_next.kept, fraction(3, 2));
    assert_eq!(receipt.member_next.gift_index, fraction(3, 4));
    assert_eq!(receipt.member_next.sharing, MicroBasisPoints::ZERO);
    assert_eq!(receipt.lanes_next[0].id, id(1));
    assert_eq!(receipt.lanes_next[0].total_earned_seen, number(28));
    assert_eq!(receipt.lanes_next[0].pool.reward_index, number(5));
    assert_eq!(receipt.lanes_next[1].total_earned_seen, number(14));
    assert_eq!(receipt.lanes_next[1].pool.reward_index, number(3));
    assert_eq!(receipt.lane_nodes_next[0].1.reward_index_seen, number(3));
    assert_eq!(receipt.lane_nodes_next[1].1.reward_index_seen, number(5));
}

#[test]
fn stake_changes_settle_all_sources_with_the_old_escrow() {
    for (stake, expected_change) in [(0, Change::Remove(1)), (3, Change::Add(2))] {
        let family = family();
        let receipt = set_credit_stake(SetCreditStakeArgs {
            protocol: family.protocol,
            parent: family.parent,
            child: family.child,
            lanes: family.lanes,
            parent_lane_nodes: family.positions,
            stake,
        })
        .unwrap();

        assert_eq!(receipt.parent_next.kept, fraction(3, 2));
        assert_eq!(receipt.parent_next.gift_index, fraction(3, 4));
        assert_eq!(receipt.child_next.kept, fraction(3, 4));
        assert_eq!(receipt.child_next.parent_gift_index_seen, fraction(3, 4));
        assert_eq!(receipt.child_next.escrow, stake);
        assert_eq!(receipt.parent_next.child_escrow, stake + 1);
        assert_eq!(receipt.escrow_change, expected_change);
        assert_eq!(receipt.lanes_next[0].total_earned_seen, number(28));
        assert_eq!(receipt.lanes_next[1].total_earned_seen, number(14));
    }
}

#[test]
fn first_stake_cannot_claim_earlier_parent_income() {
    let mut family = family();
    family.parent.child_escrow = 0;
    family.child.escrow = 0;
    let receipt = set_credit_stake(SetCreditStakeArgs {
        protocol: family.protocol,
        parent: family.parent,
        child: family.child,
        lanes: family.lanes,
        parent_lane_nodes: family.positions,
        stake: 1,
    })
    .unwrap();

    assert_eq!(receipt.parent_next.kept, number(3));
    assert_eq!(receipt.parent_next.gift_index, Number::ZERO);
    assert_eq!(receipt.child_next.kept, Number::ZERO);
    assert_eq!(receipt.child_next.escrow, 1);
    assert_eq!(receipt.parent_next.child_escrow, 1);
}

#[test]
fn family_settlement_updates_gifts_for_a_child_with_no_asset_positions() {
    let family = family();
    let lanes_before = family.lanes.clone();
    let receipt = settle_family(SettleFamilyArgs {
        protocol: &family.protocol,
        parent: family.parent,
        child: family.child,
        lanes: &family.lanes,
        parent_lane_nodes: family.positions,
    })
    .unwrap();

    assert_eq!(receipt.parent_next.kept, fraction(3, 2));
    assert_eq!(receipt.child_next.kept, fraction(3, 4));
    assert_eq!(receipt.child_next.lanes, LaneMask::default());
    assert_eq!(receipt.child_next.parent_gift_index_seen, fraction(3, 4));
    assert_eq!(
        receipt.parent_lane_nodes_next[0].1.reward_index_seen,
        number(3)
    );
    assert_eq!(
        receipt.parent_lane_nodes_next[1].1.reward_index_seen,
        number(5)
    );
    assert_eq!(family.lanes, lanes_before);
}

#[test]
fn sharing_requires_even_a_withdrawn_lanes_source() {
    let mut args = family().sharing_args();
    // Lane 1 has been withdrawn; its permanent mask still records historical income.
    args.lane_nodes[1].1.downline = 0;
    args.lanes[0].0.pool.balance = 0;
    args.lanes.remove(0);
    args.lane_nodes.remove(1);

    assert_eq!(set_sharing(args).err(), Some(Error::IncompleteSources));
}

#[test]
fn sharing_rejects_duplicate_or_unregistered_sources() {
    let mut duplicate_lane = family().sharing_args();
    duplicate_lane.lanes[0] = duplicate_lane.lanes[1].clone();
    assert_eq!(
        set_sharing(duplicate_lane).err(),
        Some(Error::IncompleteSources)
    );

    let mut duplicate_position = family().sharing_args();
    duplicate_position.lane_nodes[1] = duplicate_position.lane_nodes[0].clone();
    assert_eq!(
        set_sharing(duplicate_position).err(),
        Some(Error::IncompleteSources)
    );

    let mut unregistered_lane = family().sharing_args();
    unregistered_lane.lanes[0].0.id = id(2);
    unregistered_lane.lane_nodes[1].0 = id(2);
    assert_eq!(
        set_sharing(unregistered_lane).err(),
        Some(Error::InvalidLane)
    );
}

#[test]
fn sharing_rejects_a_regressing_total_for_either_lane() {
    for source in 0..2 {
        let mut args = family().sharing_args();
        args.lanes[source].1 = args.lanes[source]
            .0
            .total_earned_seen
            .checked_sub(Number::ONE)
            .unwrap();

        assert_eq!(set_sharing(args).err(), Some(Error::InvalidEarnings));
    }
}
