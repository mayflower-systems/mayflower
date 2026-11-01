use crate::support;
use core::{fmt::Debug, num::NonZeroU128};
use mayf_fixed_drivers::reference::DashuFixed12;
use mayf_fixed_drivers::{I64F64, U64F64, U192F64};
use mayf_fixed_number::SignedFixed;
use referral_tree::*;
use support::{Tenths, WidePico, WideTenths};

fn whole<N: FixedNumber>(value: i128) -> N {
    N::checked_from_i128(value).unwrap_or_else(|_| panic!("test integer must fit driver"))
}

fn proportion<N: FixedNumber>(numerator: i128, denominator: u128) -> N {
    N::checked_from_ratio_round(
        numerator,
        NonZeroU128::new(denominator).unwrap(),
        DirectedRounding::Floor,
    )
    .unwrap()
}

fn point_time_examples<N: FixedNumber + Debug>() {
    let config = Config {
        max_depth: 1,
        override_rate: proportion(1, 2),
    };
    let path = [
        Node {
            deposit: 100,
            downline_deposits: 100,
            child_deposits: 100,
            ..Node::<N>::default()
        },
        Node {
            deposit: 100,
            ..Node::default()
        },
    ];
    let settled = checkpoint_path(&path, CheckpointPathArgs { config, time: 10 })
        .unwrap()
        .next_path;
    assert_eq!(settled[0].point_time, whole(1500));
    assert_eq!(settled[1].point_time, whole(500));
    let shared = [
        Node {
            sharing: N::ONE,
            ..path[0]
        },
        path[1],
    ];
    let settled = checkpoint_path(&shared, CheckpointPathArgs { config, time: 10 })
        .unwrap()
        .next_path;
    assert_eq!(settled[0].point_time, N::ZERO);
    assert_eq!(settled[1].point_time, whole(2000));
    assert_eq!(
        settled[0]
            .point_time
            .checked_add(settled[1].point_time)
            .unwrap(),
        whole(2000)
    );
}

fn calculator_examples<N: FixedNumber + Debug>() {
    // Even the smallest positive represented override rate reaches the parent
    // of a one-unit deposit. No whole-unit truncation occurs in the point split.
    let tiny = Config {
        max_depth: 1,
        override_rate: N::QUANTUM,
    };
    let parent = effective_points(tiny, 0, 0, 1).unwrap();
    let child = effective_points(tiny, 1, 1, 0).unwrap();
    assert_eq!(parent, N::QUANTUM);
    assert_eq!(parent.checked_add(child).unwrap(), N::ONE);

    for count in [0, 1, 3, 7, 100] {
        for fraction in [N::ZERO, proportion(1, 3), N::ONE] {
            for available in [N::ZERO, N::QUANTUM, whole(1), whole(17)] {
                let allocation = share_points(available, fraction, count).unwrap();
                let shared = allocation
                    .index_increment
                    .checked_mul_round(whole(i128::from(count)), DirectedRounding::Floor)
                    .unwrap();
                assert_eq!(allocation.retained.checked_add(shared).unwrap(), available);
                assert!(allocation.retained >= N::ZERO);
                assert!(shared <= available);
                if count == 0 || fraction == N::ZERO {
                    assert_eq!(allocation.retained, available);
                    assert_eq!(allocation.index_increment, N::ZERO);
                }
            }
        }
    }
}

#[test]
fn conservation_on_binary_decimal_and_coarse_grids() {
    calculator_examples::<U64F64>();
    calculator_examples::<U192F64>();
    calculator_examples::<I64F64>();
    calculator_examples::<DashuFixed12>();
    calculator_examples::<Tenths>();
    calculator_examples::<WideTenths>();
    calculator_examples::<WidePico>();
    point_time_examples::<U192F64>();
    point_time_examples::<WideTenths>();
    point_time_examples::<WidePico>();
}

fn rejects_narrow<N: FixedNumber + Debug>() {
    assert_eq!(validate_point_time_range::<N>(), Err(Error::RangeTooSmall));
    let config = Config {
        max_depth: 1,
        override_rate: N::ZERO,
    };
    let path = [Node::<N>::default()];
    assert_eq!(
        checkpoint(
            path[0],
            CheckpointArgs {
                config,
                depth: 0,
                time: 1,
                parent_point_time_index: N::ZERO,
            }
        ),
        Err(Error::RangeTooSmall)
    );
    assert_eq!(
        checkpoint_path(&path, CheckpointPathArgs { config, time: 1 }),
        Err(Error::RangeTooSmall)
    );
    assert_eq!(
        set_deposit(
            &path,
            SetDepositArgs {
                config,
                time: 1,
                new_deposit: 1,
            }
        ),
        Err(Error::RangeTooSmall)
    );
    assert_eq!(
        set_sharing(
            &path,
            SetSharingArgs {
                config,
                time: 1,
                sharing: N::ZERO,
            }
        ),
        Err(Error::RangeTooSmall)
    );
    // The pure calculators have no range requirement.
    assert_eq!(effective_points(config, 0, 7, 0), Ok(whole(7)));
    assert!(share_points(N::ONE, N::ONE, 3).is_ok());
}

#[test]
fn point_time_rejects_types_too_narrow_for_the_largest_point_time() {
    rejects_narrow::<U64F64>();
    rejects_narrow::<I64F64>();
    rejects_narrow::<DashuFixed12>();
    rejects_narrow::<Tenths>();
    assert_eq!(validate_point_time_range::<U192F64>(), Ok(()));
    assert_eq!(validate_point_time_range::<WideTenths>(), Ok(()));
    assert_eq!(validate_point_time_range::<WidePico>(), Ok(()));
}

fn largest_point_time_settles<N: FixedNumber + Debug>() {
    // A full u64 of deposits over a full u64 of ticks is the worst case. The
    // parent shares everything with a one-unit child, so its index also grows
    // as fast as possible.
    let config = Config {
        max_depth: 1,
        override_rate: N::ZERO,
    };
    let path = [
        Node {
            deposit: u64::MAX - 1,
            downline_deposits: 1,
            child_deposits: 1,
            sharing: N::ONE,
            ..Node::<N>::default()
        },
        Node {
            deposit: 1,
            ..Node::default()
        },
    ];
    let settled = checkpoint_path(
        &path,
        CheckpointPathArgs {
            config,
            time: u64::MAX,
        },
    )
    .unwrap()
    .next_path;
    let largest: N = whole(i128::from(u64::MAX));
    let integral = largest
        .checked_mul_round(largest, DirectedRounding::Floor)
        .unwrap();
    assert_eq!(settled[0].point_time, N::ZERO);
    assert_eq!(settled[1].point_time, integral);
}

#[test]
fn the_largest_possible_point_time_settles_on_wide_types() {
    largest_point_time_settles::<U192F64>();
    largest_point_time_settles::<WideTenths>();
    largest_point_time_settles::<WidePico>();
}

fn rejects_negative<N: SignedFixed + Debug>() {
    let negative = N::QUANTUM.checked_neg().unwrap();
    let config = Config {
        max_depth: 1,
        override_rate: N::ZERO,
    };
    for field in 0..3 {
        let mut node = Node::default();
        match field {
            0 => node.last_parent_point_time_index = negative,
            1 => node.shared_point_time_index = negative,
            _ => node.point_time = negative,
        }
        assert_eq!(validate_node(config, 0, node), Err(Error::InvalidNode));
    }
    assert_eq!(share_points(negative, N::ONE, 1), Err(Error::InvalidAmount));
    assert_eq!(share_points(N::ZERO, negative, 0), Err(Error::InvalidRatio));
    assert_eq!(
        share_points(N::ONE, N::ONE.checked_add(N::QUANTUM).unwrap(), 1),
        Err(Error::InvalidRatio)
    );
    assert_eq!(
        validate_config(Config {
            override_rate: negative,
            ..config
        }),
        Err(Error::InvalidRatio)
    );
}

#[test]
fn signed_drivers_reject_negative_state_and_arguments() {
    rejects_negative::<I64F64>();
    rejects_negative::<DashuFixed12>();
    rejects_negative::<Tenths>();
}

fn conversion_boundary<N: FixedNumber + Debug>(largest: u64) {
    let config = Config {
        max_depth: 0,
        override_rate: N::ZERO,
    };
    assert_eq!(
        effective_points(config, 0, largest, 0),
        Ok(whole(i128::from(largest)))
    );
    if let Some(unsupported) = largest.checked_add(1) {
        assert_eq!(
            effective_points(config, 0, unsupported, 0),
            Err(Error::Overflow)
        );
    }
}

#[test]
fn integer_deposits_must_fit_the_selected_driver() {
    conversion_boundary::<U64F64>(u64::MAX);
    conversion_boundary::<U192F64>(u64::MAX);
    conversion_boundary::<I64F64>(i64::MAX as u64);
    conversion_boundary::<DashuFixed12>(u64::MAX);
    conversion_boundary::<Tenths>((i64::MAX / 10) as u64);
    conversion_boundary::<WideTenths>(u64::MAX);
}

fn accumulation_overflow<N: FixedNumber + Debug>(max: N) {
    // Only state that is already inconsistent can still overflow a wide type.
    let config = Config {
        max_depth: 1,
        override_rate: N::ZERO,
    };
    let args = CheckpointArgs {
        config,
        depth: 0,
        time: 1,
        parent_point_time_index: N::ZERO,
    };
    let retained = Node {
        deposit: 1,
        point_time: max,
        ..Node::default()
    };
    assert_eq!(checkpoint(retained, args), Err(Error::Overflow));
    assert_eq!(retained.point_time, max);
    let shared = Node {
        deposit: 1,
        downline_deposits: 1,
        child_deposits: 1,
        sharing: N::ONE,
        shared_point_time_index: max,
        ..Node::default()
    };
    assert_eq!(checkpoint(shared, args), Err(Error::Overflow));
    assert_eq!(shared.shared_point_time_index, max);
}

#[test]
fn overflow_leaves_all_inputs_unchanged() {
    accumulation_overflow(U192F64::MAX);
    accumulation_overflow(WideTenths::MAX);
    accumulation_overflow(WidePico::MAX);
}
