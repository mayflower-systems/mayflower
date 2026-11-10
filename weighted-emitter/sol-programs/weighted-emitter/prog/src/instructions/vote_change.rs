use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use mayf_weighted_emitter::{GaugeReceipt, SetGaugeVotesArgs, set_gauge_votes};

use crate::{
    clock::now_unix_seconds,
    error::WeightedEmitterProgramError,
    state::{emitter::Emitter, gauge::Gauge, numbers::Number, vote_position::VotePosition},
};

pub enum VoteChange {
    Deposit(u64),
    Withdraw(u64),
}

/// Only custody arithmetic lives here. Allocation and old-weight settlement
/// remain entirely inside the pure weighted-emitter state machine.
pub fn prepare_vote_change(
    emitter: &Account<Emitter>,
    gauge: &Account<Gauge>,
    position: &VotePosition,
    change: VoteChange,
) -> Result<(GaugeReceipt<Pubkey, Number>, u64)> {
    validate_minted_totals(emitter, gauge)?;
    require!(
        position.deposited <= gauge.total_deposited,
        WeightedEmitterProgramError::InvalidState
    );
    let (deposited, total_deposited) = match change {
        VoteChange::Deposit(amount) => {
            require!(amount > 0, WeightedEmitterProgramError::ZeroAmount);
            (
                position.deposited.checked_add(amount),
                gauge.total_deposited.checked_add(amount),
            )
        }
        VoteChange::Withdraw(amount) => {
            require!(amount > 0, WeightedEmitterProgramError::ZeroAmount);
            require!(
                amount <= position.deposited,
                WeightedEmitterProgramError::InsufficientDeposit
            );
            (
                position.deposited.checked_sub(amount),
                gauge.total_deposited.checked_sub(amount),
            )
        }
    };
    let deposited = deposited.ok_or(WeightedEmitterProgramError::ArithmeticOverflow)?;
    let total_deposited = total_deposited.ok_or(WeightedEmitterProgramError::ArithmeticOverflow)?;
    let receipt = set_gauge_votes(SetGaugeVotesArgs {
        emitter: emitter.to_library(emitter.key()),
        gauge: gauge.to_library(gauge.key()),
        votes: total_deposited,
        at: now_unix_seconds()?,
    })
    .map_err(WeightedEmitterProgramError::from)?;
    Ok((receipt, deposited))
}

pub fn validate_minted_totals(emitter: &Emitter, gauge: &Gauge) -> Result<()> {
    require!(
        emitter.total_minted <= emitter.total_emitted
            && gauge.total_minted <= gauge.total_emitted
            && gauge.total_minted <= emitter.total_minted,
        WeightedEmitterProgramError::InvalidState
    );
    Ok(())
}

pub fn apply_vote_change(
    emitter: &mut Emitter,
    gauge: &mut Gauge,
    position: &mut VotePosition,
    receipt: GaugeReceipt<Pubkey, Number>,
    deposited: u64,
) -> Result<()> {
    emitter.apply_library(receipt.emitter_next)?;
    gauge.apply_library(receipt.gauge_next);
    position.deposited = deposited;
    Ok(())
}

/// Check backing only after the token CPI and bookkeeping update. External
/// donations may increase custody, but never determine the recorded principal.
pub fn validate_escrow_backing(
    escrow: &mut Account<TokenAccount>,
    position: &VotePosition,
) -> Result<()> {
    // Anchor's deserialized token amount is stale after a token-program CPI.
    escrow.reload()?;
    require!(
        escrow.amount >= position.deposited,
        WeightedEmitterProgramError::EscrowUnderfunded
    );
    Ok(())
}
