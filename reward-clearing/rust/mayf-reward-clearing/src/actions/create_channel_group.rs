use crate::models::{Identifiable, channel_group::ChannelGroup, epoch::Epoch};
use crate::{Result, RewardClearingError, Seconds};

pub const MIN_EPOCH_DURATION_SECONDS: Seconds = 3_600;

pub struct CreateChannelGroupArgs<I: Identifiable> {
    pub channel_group_id: I,
    pub master_start_time_unix_seconds: Seconds,
    pub epoch_duration_seconds: Seconds,
}

pub struct CreateChannelGroupReceipt<I: Identifiable> {
    pub new_channel_group: ChannelGroup<I>,
    pub initial_epoch: Epoch<I>,
}

/// Create a group and epoch zero with a duration of at least one hour.
/// The initial epoch's end must fit the timestamp type.
pub fn create_channel_group<I: Identifiable>(
    args: CreateChannelGroupArgs<I>,
) -> Result<CreateChannelGroupReceipt<I>> {
    if args.epoch_duration_seconds < MIN_EPOCH_DURATION_SECONDS {
        return Err(RewardClearingError::EpochDurationTooShort);
    }

    let initial_epoch = Epoch::new(
        args.channel_group_id,
        0,
        args.master_start_time_unix_seconds,
        args.epoch_duration_seconds,
    );
    initial_epoch.end_time_unix_seconds()?;

    let new_channel_group = ChannelGroup {
        id: args.channel_group_id,
        master_start_time_unix_seconds: args.master_start_time_unix_seconds,
        epoch_duration_seconds: args.epoch_duration_seconds,
        current_epoch_index: 0,
    };

    Ok(CreateChannelGroupReceipt {
        new_channel_group,
        initial_epoch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RewardClearingError as E;
    use crate::test_support::*;

    fn create(start: Seconds, duration: Seconds) -> Result<CreateChannelGroupReceipt<Id>> {
        create_channel_group(CreateChannelGroupArgs {
            channel_group_id: CHANNEL_GROUP,
            master_start_time_unix_seconds: start,
            epoch_duration_seconds: duration,
        })
    }

    #[test]
    fn requires_at_least_one_hour() {
        for (duration, expected) in [
            (0, Some(E::EpochDurationTooShort)),
            (3_599, Some(E::EpochDurationTooShort)),
            (3_600, None),
        ] {
            assert_eq!(create(START, duration).err(), expected);
        }
    }

    #[test]
    fn accepts_the_maximum_end_but_rejects_overflow() {
        let latest_start = u64::MAX - MIN_EPOCH_DURATION_SECONDS;
        let created = create(latest_start, MIN_EPOCH_DURATION_SECONDS).unwrap();
        assert_eq!(created.initial_epoch.end_time_unix_seconds(), Ok(u64::MAX));

        let overflowing = create(latest_start + 1, MIN_EPOCH_DURATION_SECONDS);
        assert_eq!(overflowing.err(), Some(E::EpochEndTimeOverflow));
    }
}
