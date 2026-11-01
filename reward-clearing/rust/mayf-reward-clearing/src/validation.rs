//! Numeric and temporal invariants of the auction.

use crate::{
    Amount, Epoch, EpochDuration, FinalizedEpoch, Lot, Position, Result, RewardClearingError as E,
    Seconds, state::validate_principal_share_bounds,
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

fn validate_position_totals<N: FixedNumber>(
    epoch: &Epoch<N>,
    position: &Position<N>,
) -> Result<()> {
    ensure(
        position.principal <= epoch.principal && position.shares <= epoch.unclaimed_shares,
        E::PositionExceedsEpoch,
    )?;
    validate_principal_share_bounds(
        position.principal,
        position.shares,
        epoch.terms.maximum_entry_weight()?,
        E::InvalidPositionAccounting,
    )
}

fn validate_lot<N: FixedNumber>(
    epoch: &Epoch<N>,
    position: &Position<N>,
    lot: &Lot<N>,
) -> Result<()> {
    ensure(
        lot.principal <= position.principal && lot.shares <= position.shares,
        E::LotExceedsPosition,
    )?;
    ensure(
        lot.entry_weight <= epoch.terms.maximum_entry_weight()?,
        E::InvalidLotAccounting,
    )?;
    ensure(lot.principal > 0, E::EmptyLot)
}

pub(crate) fn validate_elapsed(elapsed: Seconds, duration: EpochDuration) -> Result<()> {
    ensure(elapsed <= duration.get(), E::EpochEnded)
}

pub(crate) fn validate_active_epoch<N: FixedNumber>(
    epoch: &Epoch<N>,
    now: Seconds,
) -> Result<Seconds> {
    validate_started_epoch(epoch.start, now)?;
    ensure(now < epoch.end, E::EpochEnded)?;
    Ok(now - epoch.start)
}

/// Validate numeric consistency with the supplied newest lot. Its selection
/// and the identity of the auction and participant are the host's responsibility.
pub(crate) fn validate_active_position<N: FixedNumber>(
    epoch: &Epoch<N>,
    position: &Position<N>,
    latest_lot: Option<&Lot<N>>,
    now: Seconds,
) -> Result<Seconds> {
    validate_position_totals(epoch, position)?;
    ensure(
        (position.principal == 0) == latest_lot.is_none(),
        E::InvalidPositionAccounting,
    )?;
    if let Some(lot) = latest_lot {
        validate_lot(epoch, position, lot)?;
    }
    ensure(!position.claimed, E::PositionAlreadyClaimed)?;
    validate_active_epoch(epoch, now)
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

pub(crate) fn validate_settlement<N: FixedNumber>(epoch: &Epoch<N>, now: Seconds) -> Result<()> {
    validate_ended_epoch(epoch.start, epoch.end, now)
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
