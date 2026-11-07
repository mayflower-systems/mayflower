use mayf_fixed_number::FixedNumber;

use crate::models::Identifiable;
use crate::models::channel::Channel;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::epoch::Epoch;
use crate::{Result, Seconds};

pub struct CreateChannelEpochArgs<I: Identifiable, N: FixedNumber> {
    pub channel: Channel<I, N>,
    pub now: Seconds,
    pub active_epoch: Epoch<I>,
}

pub struct CreateChannelEpochReceipt<I: Identifiable, N: FixedNumber> {
    pub channel_next: Channel<I, N>,
    pub new_channel_epoch: ChannelEpoch<I, N>,
}

/// Open a channel for bidding in the active epoch, with the channel's default terms.
/// The channel records the epoch counter, so it cannot open a second channel epoch for the same epoch.
pub fn create_channel_epoch<N, I>(
    args: CreateChannelEpochArgs<I, N>,
) -> Result<CreateChannelEpochReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    args.active_epoch.validate_active(args.now)?;
    args.channel.validate_epoch(&args.active_epoch)?;

    let new_channel_epoch = ChannelEpoch::new(
        args.channel.id,
        args.active_epoch.epoch_counter,
        args.channel.default_terms,
    );

    let channel_next = args.channel.record_channel_epoch(&args.active_epoch)?;

    Ok(CreateChannelEpochReceipt {
        channel_next,
        new_channel_epoch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RewardClearingError as E;
    use crate::test_support::*;

    fn create(
        channel: Channel<Id, Number>,
        now: Seconds,
    ) -> Result<CreateChannelEpochReceipt<Id, Number>> {
        create_channel_epoch(CreateChannelEpochArgs {
            channel,
            now,
            active_epoch: epoch(),
        })
    }

    #[test]
    fn starts_empty_with_the_channel_default_terms() {
        let created = create(channel(), HALFWAY).unwrap();
        let new_channel_epoch = &created.new_channel_epoch;

        assert_eq!(new_channel_epoch.channel_id, CHANNEL);
        assert_eq!(new_channel_epoch.epoch_counter, epoch().epoch_counter);
        assert_eq!(new_channel_epoch.terms, channel().default_terms);
        assert_eq!(new_channel_epoch.reward_pot_balance, 0);
        assert_eq!(new_channel_epoch.clearing_credits_balance, 0);
        assert_eq!(new_channel_epoch.weighted_shares_balance, shares(0));

        let channel_next = &created.channel_next;
        assert_eq!(
            channel_next.last_seen_epoch_counter,
            Some(epoch().epoch_counter)
        );
    }

    #[test]
    fn rejects_second_channel_epoch_for_the_same_epoch() {
        let created = create(channel(), START).unwrap();
        let result = create(created.channel_next, HALFWAY);
        assert_eq!(result.err(), Some(E::ChannelEpochAlreadyCreated));
    }

    #[test]
    fn rejects_creation_outside_live_interval() {
        let cases = [
            (START - 1, E::EpochNotStarted),
            (START + DURATION, E::EpochEnded),
        ];

        for (now, error) in cases {
            let result = create(channel(), now);
            assert_eq!(result.err(), Some(error));
        }
    }

    #[test]
    fn rejects_channel_from_another_channel_group() {
        let mut channel = channel();
        channel.channel_group_id = Id(8);

        let result = create(channel, HALFWAY);
        assert_eq!(result.err(), Some(E::ChannelGroupMismatch));
    }
}
