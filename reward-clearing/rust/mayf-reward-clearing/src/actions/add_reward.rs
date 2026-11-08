use crate::models::Identifiable;
use crate::models::channel::Channel;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::channel_group::ChannelGroup;
use crate::models::epoch::Epoch;
use crate::{Amount, Result, Seconds};
use mayf_fixed_number::FixedNumber;

pub struct AddRewardArgs<I: Identifiable, N: FixedNumber> {
    pub channel_group: ChannelGroup<I>,
    pub epoch: Epoch<I>,
    pub channel: Channel<I, N>,
    pub channel_epoch: ChannelEpoch<I, N>,
    pub now: Seconds,
    pub amount: Amount,
}

pub struct AddRewardReceipt<I: Identifiable, N: FixedNumber> {
    pub channel_epoch_next: ChannelEpoch<I, N>,
}

/// Add net reward to the channel epoch's pot while its epoch is live.
/// The host takes any funding fee before calling this.
pub fn add_reward<N, I>(args: AddRewardArgs<I, N>) -> Result<AddRewardReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    args.channel_epoch.validate_unfinalized()?;
    args.epoch.validate_active(&args.channel_group, args.now)?;

    args.channel_epoch
        .validate_channel_and_epoch(&args.channel, &args.epoch)?;

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
        add_with_channel(channel(), channel_epoch, now, amount)
    }

    fn add_with_channel(
        channel: Channel<Id, Number>,
        channel_epoch: ChannelEpoch<Id, Number>,
        now: Seconds,
        amount: Amount,
    ) -> Result<AddRewardReceipt<Id, Number>> {
        add_reward(AddRewardArgs {
            channel_group: channel_group(),
            epoch: epoch(),
            channel,
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
    fn rejects_channel_from_another_channel_group() {
        let mut channel = channel();
        channel.channel_group_id = Id(8);

        let result = add_with_channel(channel, channel_epoch(), START, 90);
        assert_eq!(result.err(), Some(E::ChannelGroupMismatch));
    }

    #[test]
    fn rejects_channel_epoch_from_another_channel() {
        let mut channel_epoch = channel_epoch();
        channel_epoch.channel_id = Id(7);

        let result = add(channel_epoch, START, 90);
        assert_eq!(result.err(), Some(E::ChannelEpochChannelMismatch));
    }

    #[test]
    fn rejects_channel_epoch_from_another_epoch() {
        let mut channel_epoch = channel_epoch();
        channel_epoch.epoch_counter = 4;

        let result = add(channel_epoch, START, 90);
        assert_eq!(result.err(), Some(E::ChannelEpochCounterMismatch));
    }
}
