use mayf_fixed_drivers::{I64Scaled, U64F64, U192F64, U256Scaled};

use super::*;
use crate::{MicroBasisPoints, NetworkTerms};

fn protocol(lane_count: u8) -> Protocol {
    Protocol {
        terms: NetworkTerms::new(7, MicroBasisPoints::ZERO).unwrap(),
        lane_count,
    }
}

#[test]
fn registration_skips_the_precise_starting_earnings() {
    let baseline = U192F64::from_u64(u64::MAX)
        .checked_add(
            U192F64::from_u64(1)
                .checked_div_floor(U192F64::from_u64(2))
                .unwrap(),
        )
        .unwrap();
    let receipt = register_lane(RegisterLaneArgs {
        protocol: protocol(2),
        total_earned: baseline,
    })
    .unwrap();

    assert_eq!(receipt.protocol_next, protocol(3));
    assert_eq!(receipt.new_lane.id, LaneId::new(2).unwrap());
    assert_eq!(receipt.new_lane.total_earned_seen, baseline);
    assert_eq!(receipt.new_lane.pool, LanePool::default());
}

#[test]
fn registration_rejects_invalid_inputs_and_unsuitable_number_drivers() {
    assert_eq!(
        register_lane(RegisterLaneArgs {
            protocol: protocol(MAX_LANES),
            total_earned: U192F64::ZERO,
        })
        .err(),
        Some(Error::TooManyLanes)
    );
    assert_eq!(
        register_lane(RegisterLaneArgs {
            protocol: protocol(0),
            total_earned: I64Scaled::<1>::from_scaled_i64(-1),
        })
        .err(),
        Some(Error::InvalidEarnings)
    );
    assert_eq!(
        register_lane(RegisterLaneArgs {
            protocol: protocol(0),
            total_earned: U256Scaled::<1_000_000_000_000_000_000>::ZERO,
        })
        .err(),
        Some(Error::GridTooCoarse)
    );
    assert_eq!(
        register_lane(RegisterLaneArgs {
            protocol: protocol(0),
            total_earned: U64F64::ZERO,
        })
        .err(),
        Some(Error::NumberOutOfRange)
    );
}
