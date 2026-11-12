//! Action tests start from explicit records and exercise one public transition.

mod balances;
mod families;

use alloc::vec::Vec;
use core::num::NonZeroU128;

use mayf_fixed_drivers::U192F64;
use mayf_fixed_number::{DirectedRounding::Floor, FixedNumber};

use crate::{
    Lane, LaneId, LaneMask, LaneNode, LanePool, MicroBasisPoints, NetworkTerms, Node, Offer,
    Protocol,
};

type Number = U192F64;
type Member = Node<u64, Number>;

fn number(value: u64) -> Number {
    Number::from_u64(value)
}

fn fraction(numerator: i128, denominator: u128) -> Number {
    Number::checked_from_ratio_round(numerator, NonZeroU128::new(denominator).unwrap(), Floor)
        .unwrap()
}

fn percent(value: u32) -> MicroBasisPoints {
    MicroBasisPoints::from_percent(value).unwrap()
}

fn id(slot: u8) -> LaneId {
    LaneId::new(slot).unwrap()
}

fn protocol(lane_count: u8) -> Protocol {
    Protocol {
        terms: NetworkTerms::new(2, percent(25)).unwrap(),
        lane_count,
    }
}

fn member(key: u64, parent_id: Option<u64>, lane_mask: u16) -> Member {
    Member {
        parent_id,
        depth: u8::from(parent_id.is_some()),
        lanes: LaneMask::from_bits(lane_mask),
        ..Member::top_level(
            key,
            Offer {
                current_fee: 0,
                increment: 0,
            },
        )
    }
}

fn lane(slot: u8, balance: u64, total_earned_seen: u64, index: u64) -> Lane<Number> {
    Lane {
        id: id(slot),
        total_earned_seen: number(total_earned_seen),
        pool: LanePool {
            balance,
            reward_index: number(index),
            ..LanePool::default()
        },
    }
}

fn position(balance: u64, downline: u64, index: u64) -> LaneNode<Number> {
    LaneNode {
        balance,
        downline,
        ..LaneNode::at(number(index))
    }
}

fn position_for(positions: &[(u64, LaneNode<Number>)], member_id: u64) -> &LaneNode<Number> {
    &positions.iter().find(|(id, _)| *id == member_id).unwrap().1
}

struct Family {
    protocol: Protocol,
    parent: Member,
    child: Member,
    lanes: Vec<(Lane<Number>, Number)>,
    positions: Vec<(LaneId, LaneNode<Number>)>,
}
