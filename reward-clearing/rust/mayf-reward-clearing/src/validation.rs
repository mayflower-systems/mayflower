//! Numeric and temporal invariants of the auction.

use crate::{
    Amount, EpochDuration, FinalizedEpoch, Lot, Position, Result, RewardClearingError as E,
    Seconds,
    models::{Identifiable, epoch::Epoch},
    state::validate_principal_share_bounds,
};
use mayf_fixed_number::{FixedNumber, NonNeg};

fn ensure(condition: bool, error: E) -> Result<()> {
    condition.then_some(()).ok_or(error)
}

fn validate_started_epoch(start: Seconds, now: Seconds) -> Result<()> {
    ensure(now >= start, E::TimestampBeforeEpoch)
}

fn validate_ended_epoch(start: Seconds, end: Seconds, now: Seconds) -> Result<()> {
    validate_started_epoch(start, now)?;
    ensure(now >= end, E::EpochNotEnded)
}

pub(crate) fn validate_elapsed(elapsed: Seconds, duration: EpochDuration) -> Result<()> {
    ensure(elapsed <= duration.get(), E::EpochEnded)
}

pub(crate) fn validate_epoch_is_active<I: Identifiable>(
    epoch: &Epoch<I>,
    now: Seconds,
) -> Result<()> {
    validate_started_epoch(epoch.start_time_unix_seconds, now)?;
    ensure(now < epoch.end_time_unix_seconds(), E::EpochEnded)?;
    Ok(())
}

pub(crate) fn validate_new_lot_weight<N: FixedNumber>(
    weight: NonNeg<N>,
    latest_lot: Option<&Lot<N>>,
) -> Result<()> {
    ensure(
        !latest_lot.is_some_and(|lot| weight > lot.entry_weight),
        E::LotWeightOutOfOrder,
    )
}

pub(crate) fn validate_deposit_amount<N: FixedNumber>(
    principal: Amount,
    shares: NonNeg<N>,
) -> Result<()> {
    ensure(principal > 0 && shares > NonNeg::zero(), E::DepositTooSmall)
}

pub(crate) fn validate_withdrawal_amount<N: FixedNumber>(
    principal: Amount,
    lot: &Lot<N>,
) -> Result<()> {
    ensure(principal > 0, E::WithdrawalTooSmall)?;
    ensure(principal <= lot.principal, E::WithdrawalExceedsLotPrincipal)
}

pub(crate) fn validate_claim<N: FixedNumber>(
    epoch: &FinalizedEpoch<N>,
    position: &Position<N>,
    now: Seconds,
) -> Result<()> {
    validate_ended_epoch(epoch.start, epoch.end, now)?;
    ensure(!position.claimed, E::PositionAlreadyClaimed)?;
    ensure(
        position.shares <= epoch.unclaimed_shares,
        E::PositionExceedsEpoch,
    )?;
    validate_principal_share_bounds(
        position.principal,
        position.shares,
        epoch.terms.maximum_entry_weight()?,
        E::InvalidPositionAccounting,
    )
}
