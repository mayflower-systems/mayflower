//! Core-point algebra and the limits of the splitting guarantee, as stated in
//! the crate documentation.
use crate::support;
use referral_tree::*;
use support::{Tenths, WideTenths, tenths};

const CONFIG: Config<Tenths> = Config {
    max_depth: 2,
    override_rate: Tenths::from_scaled_i64(1),
};

fn chain_points(deposits: &[u64]) -> Vec<i64> {
    let config = Config {
        max_depth: (deposits.len() - 1) as u8,
        ..CONFIG
    };
    deposits
        .iter()
        .enumerate()
        .map(|(depth, deposit)| {
            effective_points(
                config,
                depth as u8,
                *deposit,
                deposits[depth + 1..].iter().sum(),
            )
            .unwrap()
            .to_scaled_i64()
        })
        .collect()
}

#[test]
fn downward_transfers_cannot_improve_mixed_owner_core_points() {
    let deposits = [11, 7, 13, 5, 17];
    let before = chain_points(&deposits);
    for upper in 0..deposits.len() - 1 {
        for lower in upper + 1..deposits.len() {
            for amount in 1..=deposits[upper] {
                let mut moved = deposits;
                moved[upper] -= amount;
                moved[lower] += amount;
                let after = chain_points(&moved);
                for owners in 0_u32..1 << deposits.len() {
                    let owned = |i: usize| owners & (1 << i) != 0;
                    if !owned(upper) || !owned(lower) {
                        continue;
                    }
                    let mut before_owned = 0;
                    let mut after_owned = 0;
                    for i in 0..deposits.len() {
                        if owned(i) {
                            before_owned += before[i];
                            after_owned += after[i];
                        } else {
                            assert!(after[i] >= before[i], "outside node {i}");
                        }
                    }
                    assert!(after_owned <= before_owned);
                    assert_eq!(
                        after_owned == before_owned,
                        (upper + 1..lower).all(owned),
                        "only intervening outside upline members gain overrides"
                    );
                }
            }
        }
    }
}

#[test]
fn added_deposits_pay_each_upline_member_and_keep_the_depth_complement() {
    let deposits = [11, 7, 13, 5, 17];
    let before = chain_points(&deposits);
    for source in 0..deposits.len() {
        for amount in 0..=20 {
            let mut increased = deposits;
            increased[source] += amount;
            let after = chain_points(&increased);
            for i in 0..deposits.len() {
                let expected_increment = if i < source {
                    amount as i64
                } else if i == source {
                    (10 - source as i64) * amount as i64
                } else {
                    0
                };
                assert_eq!(after[i] - before[i], expected_increment);
            }
            assert_eq!(
                after.iter().sum::<i64>() - before.iter().sum::<i64>(),
                10 * amount as i64
            );
        }
    }
}

#[test]
fn new_outside_recruits_are_outside_the_existing_capital_splitting_guarantee() {
    // The owner controls the root and, in the second tree, a new intermediary.
    // The recruit brings outside capital after that upline has been chosen.
    let direct = chain_points(&[0, 100]);
    let intermediary = chain_points(&[0, 0, 100]);
    assert_eq!(direct, [100, 900]);
    assert_eq!(intermediary, [100, 100, 800]);
    assert!(intermediary[0] + intermediary[1] > direct[0]);
    assert_eq!(direct.iter().sum::<i64>(), intermediary.iter().sum::<i64>());
}

// Point-time needs a wide type; this is the same 0.1 grid as `CONFIG`.
const WIDE_CONFIG: Config<WideTenths> = Config {
    max_depth: 2,
    override_rate: tenths(1),
};

fn star_retained(parent_deposit: u64, owned_deposit: u64) -> [i64; 3] {
    let outside_deposit = 6;
    let parent = Node {
        deposit: parent_deposit,
        downline_deposits: owned_deposit + outside_deposit,
        child_deposits: owned_deposit + outside_deposit,
        sharing: WideTenths::ONE,
        ..Node::default()
    };
    let owned = Node {
        deposit: owned_deposit,
        ..Node::default()
    };
    let outside = Node {
        deposit: outside_deposit,
        ..Node::default()
    };
    let args = CheckpointPathArgs {
        config: WIDE_CONFIG,
        time: 1,
    };
    let first = checkpoint_path(&[parent, owned], args).unwrap().next_path;
    let second = checkpoint_path(&[first[0], outside], args)
        .unwrap()
        .next_path;
    [
        second[0].point_time.checked_to_scaled_u128().unwrap(),
        first[1].point_time.checked_to_scaled_u128().unwrap(),
        second[1].point_time.checked_to_scaled_u128().unwrap(),
    ]
    .map(|raw| i64::try_from(raw).unwrap())
}

#[test]
fn controlled_children_can_recapture_voluntary_gifts_without_creating_points() {
    let before_core = effective_points(CONFIG, 0, 12, 6).unwrap();
    let after_core = effective_points(CONFIG, 0, 6, 12)
        .unwrap()
        .checked_add(effective_points(CONFIG, 1, 6, 0).unwrap())
        .unwrap();
    assert_eq!(before_core, after_core);

    // Both splits are exact even on the coarse grid. Moving half the owner's
    // capital to its own child recaptures gifts formerly sent to the outsider.
    let before = star_retained(12, 0);
    let after = star_retained(6, 6);
    assert_eq!(before, [0, 0, 180]);
    assert_eq!(after, [0, 90, 90]);
    assert!(after[0] + after[1] > before[0] + before[1]);
    assert_eq!(before.iter().sum::<i64>(), after.iter().sum::<i64>());
}

#[test]
fn sharing_is_the_maximal_funded_index_with_less_than_one_index_quantum_of_bias() {
    for available_raw in 0..=125 {
        for sharing_raw in 0..=10 {
            for children in 0..=15 {
                let allocation = share_points(
                    Tenths::from_scaled_i64(available_raw),
                    Tenths::from_scaled_i64(sharing_raw),
                    children,
                )
                .unwrap();
                if children == 0 {
                    assert_eq!(allocation.retained, Tenths::from_scaled_i64(available_raw));
                    assert_eq!(allocation.index_increment, Tenths::ZERO);
                    continue;
                }
                // Integer oracle on a 0.1 grid, independent of FixedNumber.
                // Keep the ideal numerator unrounded until after division.
                let ideal_numerator = available_raw * sharing_raw;
                let denominator = 10 * children as i64;
                let expected_index_raw = ideal_numerator / denominator;
                assert_eq!(
                    allocation.index_increment,
                    Tenths::from_scaled_i64(expected_index_raw)
                );
                let distributed_raw = allocation.index_increment.to_scaled_i64() * children as i64;
                assert_eq!(
                    allocation.retained.to_scaled_i64() + distributed_raw,
                    available_raw
                );
                let donor_bias_numerator = ideal_numerator - 10 * distributed_raw;
                assert!(donor_bias_numerator >= 0);
                assert!(donor_bias_numerator < denominator);
            }
        }
    }
}
