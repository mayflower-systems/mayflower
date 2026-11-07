use crate::models::Identifiable;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::epoch::Epoch;
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
    args.epoch.validate_active(args.now)?;

    args.channel_epoch.validate_epoch(&args.epoch)?;

    let channel_epoch_next = args.channel_epoch.add_reward(args.amount)?;

    Ok(AddRewardReceipt { channel_epoch_next })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RewardClearingError as E;
    use crate::test_support::*;

    fn add(
        channel_epoch: ChannelEpoch<Id, Number>,
        now: Seconds,
        amount: Amount,
    ) -> Result<AddRewardReceipt<Id, Number>> {
        add_reward(AddRewardArgs {
            epoch: epoch(),
            channel_epoch,
            now,
            amount,
        })
    }

    #[test]
    fn adds_to_the_reward_pot() {
        let first = add(channel_epoch(), START, 90).unwrap();
        let second = add(first.channel_epoch_next, HALFWAY, 10).unwrap();
        assert_eq!(second.channel_epoch_next.reward_pot_balance, 100);
    }

    #[test]
    fn rejects_reward_after_epoch_end() {
        let result = add(channel_epoch(), START + DURATION, 90);
        assert_eq!(result.err(), Some(E::EpochEnded));
    }

    #[test]
    fn rejects_channel_epoch_from_another_epoch() {
        let mut channel_epoch = channel_epoch();
        channel_epoch.epoch_counter = 4;

        let result = add(channel_epoch, START, 90);
        assert_eq!(result.err(), Some(E::ChannelEpochCounterMismatch));
    }
}
