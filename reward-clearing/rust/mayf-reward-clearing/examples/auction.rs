//! An entire auction using accounting values and receipts alone.

use mayf_fixed_drivers::U64F64;
use mayf_reward_clearing::{
    AddRewardArgs, AddRewardState, ClaimArgs, ClaimState, DepositArgs, DepositState, EpochSchedule,
    EpochTerms, Position, SettleArgs, SettleState, StartEpochArgs, add_reward, claim, deposit,
    settle, start_epoch,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Every channel using this schedule derives the same epoch boundaries.
    let schedule = EpochSchedule::try_new(0, 100)?;
    let epoch = start_epoch(StartEpochArgs {
        window: schedule.window_at(0)?,
        terms: EpochTerms::<U64F64>::try_new(100, U64F64::ONE, 2)?,
    })?
    .epoch;

    let early = deposit(
        DepositState {
            epoch,
            position: Position::new(),
            latest_lot: None,
        },
        DepositArgs {
            principal: 100,
            now: 0,
        },
    )?;
    let later = deposit(
        DepositState {
            epoch: early.epoch,
            position: Position::new(),
            latest_lot: None,
        },
        DepositArgs {
            principal: 100,
            now: 50,
        },
    )?;
    let funded = add_reward(
        AddRewardState { epoch: later.epoch },
        AddRewardArgs {
            amount: 1_000,
            now: 50,
        },
    )?;
    let settled = settle(
        SettleState {
            epoch: funded.epoch,
        },
        SettleArgs { now: 100 },
    )?;
    assert_eq!(settled.principal_forfeited, 200);

    let early_claim = claim(
        ClaimState {
            epoch: settled.epoch,
            position: early.position,
        },
        ClaimArgs { now: 100 },
    )?;
    let later_claim = claim(
        ClaimState {
            epoch: early_claim.epoch,
            position: later.position,
        },
        ClaimArgs { now: 100 },
    )?;
    let remainder = later_claim.epoch.remaining_reward()?;
    assert_eq!(
        (
            early_claim.reward_earned,
            later_claim.reward_earned,
            remainder
        ),
        (571, 428, 1)
    );
    assert_eq!(
        early_claim.reward_earned + later_claim.reward_earned + remainder,
        1_000
    );
    println!("Early participant: 571; later participant: 428; remainder: 1.");
    Ok(())
}
