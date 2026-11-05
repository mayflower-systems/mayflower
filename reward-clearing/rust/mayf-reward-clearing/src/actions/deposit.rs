use crate::{
    Amount, Epoch, EpochDuration, Lot, Position, RampBonusSlope, Result, RewardClearingError,
    Seconds,
    amount::from_amount,
    math::{arithmetic, ratio},
    validation::{
        validate_active_position, validate_deposit_amount, validate_elapsed,
        validate_new_lot_weight,
    },
};
use mayf_fixed_number::{DirectedRounding, FixedNumber, NonNeg};

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

    arithmetic(NonNeg::<N>::one().checked_add(bonus))
}
