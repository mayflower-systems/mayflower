use mayf_fixed_number::FixedNumber;

use crate::models::Identifiable;
use crate::models::channel::Channel;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::channel_group::ChannelGroup;
use crate::models::epoch::Epoch;
use crate::{Result, RewardClearingError, Seconds};

pub struct CreateChannelEpochArgs<I: Identifiable, N: FixedNumber> {
    pub channel: Channel<I, N>,
    pub channel_group: ChannelGroup<I>,
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
    if args.channel.disabled {
        return Err(RewardClearingError::ChannelDisabled);
    }

    args.active_epoch
        .validate_active(&args.channel_group, args.now)?;
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

    const NEXT_START: Seconds = START + DURATION;

    fn create(
        channel: Channel<Id, Number>,
        now: Seconds,
    ) -> Result<CreateChannelEpochReceipt<Id, Number>> {
        let mut group = channel_group();
        group.current_epoch_index += 1;
        let active_epoch = Epoch::new(group.id, group.current_epoch_index, NEXT_START, DURATION);
        create_channel_epoch(CreateChannelEpochArgs {
            channel,
            channel_group: group,
            now,
            active_epoch,
        })
    }

    #[test]
    fn rejects_disabled_channel() {
        let mut channel = channel();
        channel.disabled = true;

        assert_eq!(create(channel, NEXT_START).err(), Some(E::ChannelDisabled));
    }

    #[test]
    fn requires_the_channel_groups_current_epoch() {
        let mut another_group = channel_group();
        another_group.id = Id(8);
        another_group.current_epoch_index = 4;
        let mut later_epoch = channel_group();
        later_epoch.current_epoch_index = 5;

        for (channel_group, expected) in [
            (another_group, E::EpochChannelGroupMismatch),
            (later_epoch, E::ChannelGroupCounterMismatch),
        ] {
            let result = create_channel_epoch(CreateChannelEpochArgs {
                channel: channel(),
                channel_group,
                active_epoch: Epoch::new(CHANNEL_GROUP, 4, NEXT_START, DURATION),
                now: NEXT_START,
            });
            assert_eq!(result.err(), Some(expected));
        }
    }

    #[test]
    fn opens_later_epochs_with_the_current_default_terms() {
        for counter in [4, 6] {
            let mut channel = channel();
            channel.default_terms = terms(2, 3);
            let mut group = channel_group();
            group.current_epoch_index = counter;
            let start = START + u64::from(counter - 3) * DURATION;
            let created = create_channel_epoch(CreateChannelEpochArgs {
                channel,
                channel_group: group,
                active_epoch: Epoch::new(CHANNEL_GROUP, counter, start, DURATION),
                now: start,
            })
            .unwrap();

            let new_channel_epoch = created.new_channel_epoch;
            assert_eq!(new_channel_epoch.epoch_counter, counter);
            assert_eq!(new_channel_epoch.terms, terms(2, 3));
            assert_eq!(new_channel_epoch.reward_pot_balance, 0);
            assert_eq!(new_channel_epoch.clearing_credits_balance, 0);
            assert_eq!(new_channel_epoch.weighted_shares_balance, shares(0));
            assert_eq!(created.channel_next.last_created_epoch_counter, counter);
        }
    }

    #[test]
    fn rejects_second_channel_epoch_for_the_same_epoch() {
        let created = create(channel(), NEXT_START).unwrap();
        let result = create(created.channel_next, NEXT_START + DURATION / 2);
        assert_eq!(result.err(), Some(E::ChannelEpochAlreadyCreated));
    }

    #[test]
    fn rejects_creation_outside_live_interval() {
        let cases = [
            (NEXT_START - 1, E::EpochNotStarted),
            (NEXT_START + DURATION, E::EpochEnded),
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

        let result = create(channel, NEXT_START);
        assert_eq!(result.err(), Some(E::ChannelGroupMismatch));
    }
}
