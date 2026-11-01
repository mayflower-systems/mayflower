//! Pure auction transitions: numeric state in, replacement state and amounts out.

use crate::{
    Amount, Epoch, EpochDuration, EpochTerms, EpochWindow, ExitPenaltyExponent, FinalizedEpoch,
    Lot, Position, RampBonusSlope, Result, RewardClearingError, Seconds,
    amount::from_amount,
    validation::{
        validate_active_epoch, validate_active_position, validate_claim, validate_deposit_amount,
        validate_elapsed, validate_new_lot_weight, validate_settlement, validate_withdrawal_amount,
    },
};
use core::num::NonZeroU128;
use mayf_fixed_number::{DirectedRounding, FixedNumber, IntegralRounding, MathResult, NonNeg, Pos};

fn arithmetic<T>(result: MathResult<T>) -> Result<T> {
    result.map_err(|_| RewardClearingError::ArithmeticFailure)
}

fn one<N: FixedNumber>() -> NonNeg<N> {
    NonNeg::one()
}

fn positive<N: FixedNumber>(value: NonNeg<N>) -> Result<Pos<N>> {
    arithmetic(Pos::try_from(value))
}

fn ratio<N: FixedNumber>(
    numerator: u64,
    denominator: EpochDuration,
    rounding: DirectedRounding,
) -> Result<NonNeg<N>> {
    let denominator = NonZeroU128::from(core::num::NonZeroU64::from(denominator));
    let value = arithmetic(N::checked_from_ratio_round(
        i128::from(numerator),
        denominator,
        rounding,
    ))?;
    arithmetic(NonNeg::new(value))
}

/// Compute the fixed entry weight for an elapsed epoch time and bonus slope.
///
/// The result is `1 + slope * (1 - elapsed / duration)`, conservatively
/// rounded down. It starts at `1 + slope` and ends at the baseline `1`.
pub fn entry_weight<N: FixedNumber>(
    elapsed: Seconds,
    duration: EpochDuration,
    ramp_bonus_slope: RampBonusSlope<N>,
) -> Result<NonNeg<N>> {
    validate_elapsed(elapsed, duration)?;

    let remaining = ratio(duration.get() - elapsed, duration, DirectedRounding::Floor)?;
    let bonus = arithmetic(ramp_bonus_slope.get().checked_mul_floor(remaining))?;

    arithmetic(one::<N>().checked_add(bonus))
}

/// Compute the fixed power-curve withdrawal penalty for an elapsed epoch time.
///
/// The result is `(elapsed / duration)^exponent`, conservatively rounded up:
/// `0` at the start and `1` at the end for every positive exponent.
pub fn withdrawal_penalty<N: FixedNumber>(
    elapsed: Seconds,
    duration: EpochDuration,
    exit_penalty_exponent: ExitPenaltyExponent,
) -> Result<NonNeg<N>> {
    validate_elapsed(elapsed, duration)?;

    let progress: NonNeg<N> = ratio(elapsed, duration, DirectedRounding::Ceil)?;
    let penalty = arithmetic(progress.get().checked_pow_ceil(exit_penalty_exponent.get()))?;
    let penalty = arithmetic(NonNeg::new(penalty))?;

    Ok(penalty.min(one()))
}

/// Immutable schedule window and terms for a new auction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartEpochArgs<N: FixedNumber> {
    pub window: EpochWindow,
    pub terms: EpochTerms<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartEpochReceipt<N: FixedNumber> {
    pub epoch: Epoch<N>,
}

pub fn start_epoch<N: FixedNumber>(args: StartEpochArgs<N>) -> Result<StartEpochReceipt<N>> {
    Ok(StartEpochReceipt {
        epoch: Epoch::try_new(args.window, args.terms)?,
    })
}

/// The host supplies totals and the current newest lot from the same position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DepositState<N: FixedNumber> {
    pub epoch: Epoch<N>,
    pub position: Position<N>,
    pub latest_lot: Option<Lot<N>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DepositArgs {
    pub principal: Amount,
    pub now: Seconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DepositReceipt<N: FixedNumber> {
    pub principal_deposited: Amount,
    pub shares_issued: NonNeg<N>,
    pub entry_weight: NonNeg<N>,
    pub epoch: Epoch<N>,
    pub position: Position<N>,
    /// New lot to append after the supplied newest lot.
    pub lot: Lot<N>,
}

/// Issue shares for a new deposit without changing previous lots' weights.
pub fn deposit<N: FixedNumber>(
    state: DepositState<N>,
    args: DepositArgs,
) -> Result<DepositReceipt<N>> {
    let DepositState {
        epoch,
        position,
        latest_lot,
    } = state;
    let elapsed = validate_active_position(&epoch, &position, latest_lot.as_ref(), args.now)?;
    let entry_weight = entry_weight(elapsed, epoch.duration(), epoch.ramp_bonus_slope())?;
    validate_new_lot_weight(entry_weight, latest_lot.as_ref())?;
    let shares_issued =
        arithmetic(from_amount::<N>(args.principal)?.checked_mul_floor(entry_weight))?;
    validate_deposit_amount(args.principal, shares_issued)?;
    let next_position = Position {
        principal: position
            .principal
            .checked_add(args.principal)
            .ok_or(RewardClearingError::ArithmeticFailure)?,
        shares: arithmetic(position.shares.checked_add(shares_issued))?,
        ..position
    };
    let next_epoch = Epoch {
        principal: epoch
            .principal
            .checked_add(args.principal)
            .ok_or(RewardClearingError::ArithmeticFailure)?,
        unclaimed_shares: arithmetic(epoch.unclaimed_shares.checked_add(shares_issued))?,
        ..epoch
    };
    Ok(DepositReceipt {
        principal_deposited: args.principal,
        shares_issued,
        entry_weight,
        epoch: next_epoch,
        position: next_position,
        lot: Lot {
            principal: args.principal,
            shares: shares_issued,
            entry_weight,
        },
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddRewardState<N: FixedNumber> {
    pub epoch: Epoch<N>,
}

/// Net reward credited to this auction. Any funding fee is a host policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddRewardArgs {
    pub amount: Amount,
    pub now: Seconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddRewardReceipt<N: FixedNumber> {
    pub reward_to_epoch: Amount,
    pub epoch: Epoch<N>,
}

/// Increase net reward while the auction is active. Reject a pool that the
/// selected driver could not widen for a later entitlement calculation.
pub fn add_reward<N: FixedNumber>(
    state: AddRewardState<N>,
    args: AddRewardArgs,
) -> Result<AddRewardReceipt<N>> {
    validate_active_epoch(&state.epoch, args.now)?;
    let unclaimed_reward = state
        .epoch
        .unclaimed_reward
        .checked_add(args.amount)
        .ok_or(RewardClearingError::ArithmeticFailure)?;
    from_amount::<N>(unclaimed_reward)?;
    Ok(AddRewardReceipt {
        reward_to_epoch: args.amount,
        epoch: Epoch {
            unclaimed_reward,
            ..state.epoch
        },
    })
}

/// The host supplies the authenticated current newest lot from this position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WithdrawState<N: FixedNumber> {
    pub epoch: Epoch<N>,
    pub position: Position<N>,
    pub lot: Lot<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WithdrawArgs {
    pub principal: Amount,
    pub now: Seconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WithdrawReceipt<N: FixedNumber> {
    pub principal_withdrawn: Amount,
    pub principal_returned: Amount,
    pub principal_forfeited: Amount,
    pub shares_removed: NonNeg<N>,
    pub penalty: NonNeg<N>,
    pub epoch: Epoch<N>,
    pub position: Position<N>,
    /// `None` means the current lot has been completely consumed.
    pub lot: Option<Lot<N>>,
}

/// Remove principal from one lot, conserving its return/forfeiture split and
/// removing proportional shares. A request spanning lots repeats this action
/// with each newly exposed newest lot.
#[inline(never)]
pub fn withdraw<N: IntegralRounding>(
    state: WithdrawState<N>,
    args: WithdrawArgs,
) -> Result<WithdrawReceipt<N>> {
    let WithdrawState {
        epoch,
        position,
        lot,
    } = state;
    let elapsed = validate_active_position(&epoch, &position, Some(&lot), args.now)?;
    validate_withdrawal_amount(args.principal, &lot)?;
    let principal = args.principal;
    let shares_removed = if principal == lot.principal {
        lot.shares
    } else {
        let lot_principal = positive(from_amount::<N>(lot.principal)?)?;
        arithmetic(from_amount::<N>(principal)?.checked_mul_div_ceil(lot.shares, lot_principal))?
            .min(lot.shares)
    };
    let penalty = withdrawal_penalty(elapsed, epoch.duration(), epoch.exit_penalty_exponent())?;
    let return_fraction = arithmetic(one::<N>().checked_sub(penalty))?;
    let fractional_return =
        arithmetic(from_amount::<N>(principal)?.checked_mul_floor(return_fraction))?;
    let returned = arithmetic(fractional_return.get().checked_floor_to_u64())?;
    let principal_forfeited = principal
        .checked_sub(returned)
        .ok_or(RewardClearingError::ArithmeticFailure)?;
    let remaining_principal = lot
        .principal
        .checked_sub(principal)
        .ok_or(RewardClearingError::ArithmeticFailure)?;
    let remaining_shares = arithmetic(lot.shares.checked_sub(shares_removed))?;
    let next_lot = if remaining_principal > 0 {
        Some(Lot {
            principal: remaining_principal,
            shares: remaining_shares,
            ..lot
        })
    } else {
        None
    };
    let next_position = Position {
        principal: position
            .principal
            .checked_sub(principal)
            .ok_or(RewardClearingError::ArithmeticFailure)?,
        shares: arithmetic(position.shares.checked_sub(shares_removed))?,
        ..position
    };
    let next_epoch = Epoch {
        principal: epoch
            .principal
            .checked_sub(principal)
            .ok_or(RewardClearingError::ArithmeticFailure)?,
        unclaimed_shares: arithmetic(epoch.unclaimed_shares.checked_sub(shares_removed))?,
        ..epoch
    };
    Ok(WithdrawReceipt {
        principal_withdrawn: principal,
        principal_returned: returned,
        principal_forfeited,
        shares_removed,
        penalty,
        epoch: next_epoch,
        position: next_position,
        lot: next_lot,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettleState<N: FixedNumber> {
    pub epoch: Epoch<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettleArgs {
    pub now: Seconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettleReceipt<N: FixedNumber> {
    /// Tracked principal irreversibly forfeited. Unsolicited surplus is not
    /// part of the auction and is handled by its host.
    pub principal_forfeited: Amount,
    pub epoch: FinalizedEpoch<N>,
}

/// Forfeit remaining tracked principal and freeze the original settlement
/// numerator and denominator. Later claims never divide mutable leftovers.
pub fn settle<N: FixedNumber>(state: SettleState<N>, args: SettleArgs) -> Result<SettleReceipt<N>> {
    let epoch = state.epoch;
    validate_settlement(&epoch, args.now)?;
    Ok(SettleReceipt {
        principal_forfeited: epoch.principal,
        epoch: FinalizedEpoch {
            master_start: epoch.master_start,
            epoch_counter: epoch.epoch_counter,
            start: epoch.start,
            end: epoch.end,
            terms: epoch.terms,
            settlement_shares: epoch.unclaimed_shares,
            settlement_reward: epoch.unclaimed_reward,
            unclaimed_shares: epoch.unclaimed_shares,
            unclaimed_reward: epoch.unclaimed_reward,
        },
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimState<N: FixedNumber> {
    pub epoch: FinalizedEpoch<N>,
    pub position: Position<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimArgs {
    pub now: Seconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimReceipt<N: FixedNumber> {
    pub shares_claimed: NonNeg<N>,
    /// Whole reward earned by this position. The host chooses how to credit
    /// or transfer the amount and enforces replay protection in its state.
    pub reward_earned: Amount,
    pub epoch: FinalizedEpoch<N>,
    pub position: Position<N>,
}

fn calculate_claim_reward<N: IntegralRounding>(
    epoch: &FinalizedEpoch<N>,
    shares: NonNeg<N>,
) -> Result<Amount> {
    if shares == NonNeg::zero() {
        return Ok(0);
    }
    let divisor = positive(epoch.settlement_shares)?;
    let pro_rata = arithmetic(
        from_amount::<N>(epoch.settlement_reward)?.checked_mul_div_floor(shares, divisor),
    )?;
    arithmetic(pro_rata.get().checked_floor_to_u64())
}

/// Claim once at the immutable settlement ratio, flooring the entitlement to
/// whole units. Unpaid units stay in the auction, never in a fractional user
/// balance. All eligible shares are retired even when the payout is zero.
pub fn claim<N: IntegralRounding>(
    state: ClaimState<N>,
    args: ClaimArgs,
) -> Result<ClaimReceipt<N>> {
    let ClaimState { epoch, position } = state;
    validate_claim(&epoch, &position, args.now)?;
    let reward_earned = calculate_claim_reward(&epoch, position.shares)?;
    let unclaimed_shares = arithmetic(epoch.unclaimed_shares.checked_sub(position.shares))?;
    let unclaimed_reward = epoch
        .unclaimed_reward
        .checked_sub(reward_earned)
        .ok_or(RewardClearingError::ArithmeticFailure)?;
    Ok(ClaimReceipt {
        shares_claimed: position.shares,
        reward_earned,
        epoch: FinalizedEpoch {
            unclaimed_shares,
            unclaimed_reward,
            ..epoch
        },
        position: Position {
            claimed: true,
            ..position
        },
    })
}
