use crate::{Amount, Epoch, FinalizedEpoch, Result, Seconds, validation::validate_settlement};
use mayf_fixed_number::FixedNumber;

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
