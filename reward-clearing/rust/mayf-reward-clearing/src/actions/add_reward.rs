use crate::models::Identifiable;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::epoch::Epoch;
use crate::validation::validate_epoch_is_active;
use crate::{Amount, Result, Seconds};
use mayf_fixed_number::FixedNumber;

pub struct AddRewardArgs<I: Identifiable, N: FixedNumber> {
    pub epoch: Epoch<I>,
    pub channel_epoch: ChannelEpoch<I, N>,
    pub now: Seconds,
    pub amount: Amount,
}

pub struct AddRewardReceipt<I: Identifiable, N: FixedNumber> {
    pub channel_epoch_next: ChannelEpoch<I, N>,
}

/// Increase net reward while the auction is active. Reject a pool that the
/// selected driver could not widen for a later entitlement calculation.
pub fn add_reward<N, I>(args: AddRewardArgs<I, N>) -> Result<AddRewardReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    validate_epoch_is_active(&args.epoch, args.now)?;

    args.channel_epoch.validate_epoch(&args.epoch)?;

    let channel_epoch_next = args.channel_epoch.add_reward(args.amount)?;

    Ok(AddRewardReceipt { channel_epoch_next })
}
