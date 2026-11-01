//! Independent source attribution and arbitrary-precision grid arithmetic.
//! The reference rebuilds subtree totals after each edit; it shares no formula
//! helpers or fixed-point arithmetic with the implementation under test.
use core::fmt::Debug;
use mayf_fixed_drivers::reference::UBig;
use mayf_fixed_drivers::{U64F64, U192F64};
use referral_tree::{
    CheckpointPathArgs, Config, FixedNumber, Node, SetDepositArgs, SetSharingArgs, checkpoint_path,
    set_deposit, set_sharing,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Reference {
    deposit: u64,
    downline_deposits: u64,
    child_deposits: u64,
    sharing: u128,
    last_time: u64,
    last_parent_point_time_index: u128,
    shared_point_time_index: u128,
    point_time: u128,
}

fn snapshot<N: FixedNumber>(node: Node<N>, raw: &impl Fn(N) -> u128) -> Reference {
    Reference {
        deposit: node.deposit,
        downline_deposits: node.downline_deposits,
        child_deposits: node.child_deposits,
        sharing: raw(node.sharing),
        last_time: node.last_time,
        last_parent_point_time_index: raw(node.last_parent_point_time_index),
        shared_point_time_index: raw(node.shared_point_time_index),
        point_time: raw(node.point_time),
    }
}

const PARENTS: [Option<usize>; 7] = [None, Some(0), Some(0), Some(1), Some(1), Some(2), Some(3)];
const DEPTHS: [u8; 7] = [0, 1, 1, 2, 2, 2, 3];

fn path_to(mut source: usize) -> Vec<usize> {
    let mut path = vec![source];
    while let Some(parent) = PARENTS[source] {
        path.push(parent);
        source = parent;
    }
    path.reverse();
    path
}

fn read_path<N: FixedNumber>(nodes: &[Node<N>], ids: &[usize]) -> Vec<Node<N>> {
    ids.iter().map(|i| nodes[*i]).collect()
}

fn write_path<N: FixedNumber>(nodes: &mut [Node<N>], ids: &[usize], replacement: &[Node<N>]) {
    for (id, node) in ids.iter().zip(replacement) {
        nodes[*id] = *node;
    }
}

fn recompute(nodes: &mut [Reference]) {
    for node in nodes.iter_mut() {
        node.downline_deposits = 0;
        node.child_deposits = 0;
    }
    for source in 0..nodes.len() {
        let own = nodes[source].deposit;
        if let Some(parent) = PARENTS[source] {
            nodes[parent].child_deposits += own;
        }
        let mut upline = PARENTS[source];
        while let Some(i) = upline {
            nodes[i].downline_deposits += own;
            upline = PARENTS[i];
        }
    }
}

fn reference_checkpoint(
    nodes: &mut [Reference],
    ids: &[usize],
    time: u64,
    scale: u128,
    override_rate: u128,
) {
    let mut weights = [0_u128; 7];
    // Derive allocations from each source, independently of cached subtrees.
    for source in 0..nodes.len() {
        let own = u128::from(nodes[source].deposit);
        weights[source] += own * (scale - u128::from(DEPTHS[source]) * override_rate);
        let mut upline = PARENTS[source];
        while let Some(i) = upline {
            weights[i] += own * override_rate;
            upline = PARENTS[i];
        }
    }
    for &i in ids {
        let parent_index = PARENTS[i].map_or(0, |p| nodes[p].shared_point_time_index);
        let n = &mut nodes[i];
        let elapsed = time - n.last_time;
        let earned = weights[i] * u128::from(elapsed);
        let received = u128::from(n.deposit) * (parent_index - n.last_parent_point_time_index);
        let available = earned + received;
        let increment = if n.child_deposits == 0 {
            0
        } else {
            u128::try_from(
                UBig::from(available) * UBig::from(n.sharing)
                    / (UBig::from(scale) * UBig::from(n.child_deposits)),
            )
            .unwrap()
        };
        let donated = increment * u128::from(n.child_deposits);
        assert!(
            UBig::from(donated) * UBig::from(scale)
                <= UBig::from(available) * UBig::from(n.sharing)
        );
        n.point_time += available - donated;
        n.shared_point_time_index += increment;
        n.last_time = time;
        n.last_parent_point_time_index = parent_index;
    }
}

fn changing_history<N: FixedNumber + Debug>(value: impl Fn(u128) -> N, raw: impl Fn(N) -> u128) {
    let scale = raw(N::ONE);
    let mut rng = 71091829_u64;
    for numerator in [0, 1, 2, 3] {
        // Policy values use the driver's grid. Binary and decimal need not
        // approximate the same rational to the same precision.
        let override_rate = scale * numerator / 13;
        let config = Config {
            max_depth: 3,
            override_rate: value(override_rate),
        };
        let mut lazy = vec![Node::<N>::default(); 7];
        let mut reference = vec![Reference::default(); 7];
        let mut time = 0_u64;
        let mut integrated = 0_u128;
        for step in 0..3000 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let delta = rng % 19;
            let total_points: u128 = reference.iter().map(|n| u128::from(n.deposit)).sum();
            integrated += u128::from(delta) * total_points * scale;
            time += delta;
            let source = (rng / 19 % 7) as usize;
            let ids = path_to(source);
            let path = read_path(&lazy, &ids);
            reference_checkpoint(&mut reference, &ids, time, scale, override_rate);
            match rng / 133 % 4 {
                0 | 1 => {
                    let own = rng / 532 % 31;
                    let result = set_deposit(
                        &path,
                        SetDepositArgs {
                            config,
                            time,
                            new_deposit: own,
                        },
                    )
                    .unwrap()
                    .next_path;
                    write_path(&mut lazy, &ids, &result);
                    reference[source].deposit = own;
                    recompute(&mut reference);
                }
                2 => {
                    let sharing = scale * u128::from(rng / 532 % 8) / 7;
                    let result = set_sharing(
                        &path,
                        SetSharingArgs {
                            config,
                            time,
                            sharing: value(sharing),
                        },
                    )
                    .unwrap()
                    .next_path;
                    write_path(&mut lazy, &ids, &result);
                    reference[source].sharing = sharing;
                }
                _ => {
                    let result = checkpoint_path(&path, CheckpointPathArgs { config, time })
                        .unwrap()
                        .next_path;
                    write_path(&mut lazy, &ids, &result);
                }
            }
            assert_eq!(
                lazy.iter()
                    .copied()
                    .map(|n| snapshot(n, &raw))
                    .collect::<Vec<_>>(),
                reference,
                "override numerator={numerator} step={step}"
            );
            if step % 137 == 0 || step == 2999 {
                for source in 0..lazy.len() {
                    let ids = path_to(source);
                    let replacement = checkpoint_path(
                        &read_path(&lazy, &ids),
                        CheckpointPathArgs { config, time },
                    )
                    .unwrap()
                    .next_path;
                    write_path(&mut lazy, &ids, &replacement);
                    reference_checkpoint(&mut reference, &ids, time, scale, override_rate);
                }
                assert_eq!(
                    lazy.iter()
                        .copied()
                        .map(|n| snapshot(n, &raw))
                        .collect::<Vec<_>>(),
                    reference
                );
                // Full settlement materializes every outstanding child right.
                // No base fraction is lost: elapsed ticks are integers.
                let retained: u128 = lazy.iter().map(|n| raw(n.point_time)).sum();
                assert_eq!(retained, integrated);
            }
        }
    }
}

// Point-time actions accept only types wide enough for the largest
// point-time, so the histories run on the wide drivers.
use crate::support;

#[test]
fn binary_history_matches_independent_path_accounting() {
    changing_history(
        |raw| U192F64::from(U64F64::from_scaled_u128(raw)),
        |value| U64F64::try_from(value).unwrap().to_scaled_u128(),
    );
}

#[test]
fn decimal_history_matches_independent_path_accounting() {
    changing_history(support::WidePico::from_scaled_u128, |value| {
        value.checked_to_scaled_u128().unwrap()
    });
}

#[test]
fn coarse_grid_history_matches_independent_path_accounting() {
    changing_history(support::WideTenths::from_scaled_u128, |value| {
        value.checked_to_scaled_u128().unwrap()
    });
}
