//! Run: cargo run -p referral-tree --example testbench
use core::num::NonZeroU128;
use mayf_fixed_drivers::{U64F64, U192F64};
use referral_tree::{
    CheckpointPathArgs, Config, DirectedRounding, FixedNumber, Node, SetDepositArgs,
    SetSharingArgs, checkpoint_path, set_deposit, set_sharing,
};

fn units(value: i128) -> U192F64 {
    U192F64::checked_from_i128(value).unwrap()
}

fn fraction(numerator: i128, denominator: u128) -> U192F64 {
    U192F64::checked_from_ratio_round(
        numerator,
        NonZeroU128::new(denominator).unwrap(),
        DirectedRounding::Ceil,
    )
    .unwrap()
}

/// Format a small value with three decimal places.
fn show(value: U192F64) -> String {
    let raw = U64F64::try_from(value).unwrap().to_scaled_u128();
    let thousandths = ((raw & u128::from(u64::MAX)) * 1000) >> 64;
    format!("{}.{thousandths:03}", raw >> 64)
}

fn total(path: &[Node<U192F64>]) -> U192F64 {
    path.iter().fold(U192F64::ZERO, |sum, node| {
        sum.checked_add(node.point_time).unwrap()
    })
}

fn main() {
    let config = Config {
        max_depth: 2,
        // 12.5% is exact on U192F64's binary grid; 10% would not be.
        override_rate: fraction(1, 8),
    };
    // Alice -> Bob -> Charlie, each depositing 100. The example owns storage;
    // the library returns replacement paths. One clock tick is one hour here.
    let mut path = vec![Node::<U192F64>::default(); 3];
    for source in 0..3 {
        let next = set_deposit(
            &path[..=source],
            SetDepositArgs {
                config,
                time: 0,
                new_deposit: 100,
            },
        )
        .unwrap()
        .next_path;
        path[..=source].copy_from_slice(&next);
    }
    path = checkpoint_path(&path, CheckpointPathArgs { config, time: 10 })
        .unwrap()
        .next_path;
    println!("After ten hours, with a 12.5% override rate:");
    for (name, node) in ["Alice", "Bob", "Charlie"].iter().zip(&path) {
        println!("  {name}: {} point-hours", show(node.point_time));
    }
    assert_eq!(total(&path), units(3000));

    // Bob starts sharing 50% with Charlie. His previous point-time stays his.
    let next = set_sharing(
        &path[..2],
        SetSharingArgs {
            config,
            time: 10,
            sharing: fraction(1, 2),
        },
    )
    .unwrap()
    .next_path;
    path[..2].copy_from_slice(&next);
    path = checkpoint_path(&path, CheckpointPathArgs { config, time: 20 })
        .unwrap()
        .next_path;
    println!("After ten more hours, with Bob sharing 50%:");
    for (name, node) in ["Alice", "Bob", "Charlie"].iter().zip(&path) {
        println!("  {name}: {} cumulative point-hours", show(node.point_time));
    }
    assert_eq!(total(&path), units(6000));
    assert_eq!(path[0].point_time, units(2500));
    assert_eq!(path[1].point_time, units(1500));
    assert_eq!(path[2].point_time, units(2000));
}
