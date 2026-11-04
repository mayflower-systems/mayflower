use crate::{
    Amount, Epoch, EpochDuration, ExitPenaltyExponent, Lot, Position, Result, RewardClearingError,
    Seconds,
    amount::from_amount,
    math::{arithmetic, positive, ratio},
    validation::{validate_active_position, validate_elapsed, validate_withdrawal_amount},
};
use mayf_fixed_number::{DirectedRounding, FixedNumber, IntegralRounding, NonNeg};

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
    let return_fraction = arithmetic(NonNeg::<N>::one().checked_sub(penalty))?;
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

    Ok(penalty.min(NonNeg::one()))
}
