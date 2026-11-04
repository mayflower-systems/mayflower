use crate::{
    Amount, Epoch, Result, RewardClearingError, Seconds, amount::from_amount,
    validation::validate_active_epoch,
};
use mayf_fixed_number::FixedNumber;

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
