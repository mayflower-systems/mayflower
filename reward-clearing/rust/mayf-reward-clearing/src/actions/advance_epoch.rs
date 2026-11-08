use crate::{
    Result, RewardClearingError, Seconds,
    models::{Identifiable, channel_group::ChannelGroup, epoch::Epoch},
};

pub struct AdvanceEpochArgs<I: Identifiable> {
    pub now: Seconds,
    pub channel_group: ChannelGroup<I>,
    pub current_epoch: Epoch<I>,
}

pub struct AdvanceEpochReceipt<I: Identifiable> {
    pub new_epoch: Epoch<I>,
    pub channel_group_next: ChannelGroup<I>,
}

pub fn advance_epoch<I>(args: AdvanceEpochArgs<I>) -> Result<AdvanceEpochReceipt<I>>
where
    I: Identifiable,
{
    if !args.current_epoch.is_finished(args.now)? {
        return Err(RewardClearingError::EpochNotEnded);
    }

    args.current_epoch.validate_current(&args.channel_group)?;

    // the new epoch starts at the end time of the current epoch
    let start_time_unix_seconds = args.current_epoch.end_time_unix_seconds()?;

    let next_epoch_counter = args.channel_group.next_epoch_index()?;

    let new_epoch = Epoch::new(
        args.channel_group.id,
        next_epoch_counter,
        start_time_unix_seconds,
        args.channel_group.epoch_duration_seconds,
    );
    new_epoch.end_time_unix_seconds()?;

    let channel_group_next = ChannelGroup {
        id: args.channel_group.id,
        master_start_time_unix_seconds: args.channel_group.master_start_time_unix_seconds,
        epoch_duration_seconds: args.channel_group.epoch_duration_seconds,
        current_epoch_index: next_epoch_counter,
    };

    Ok(AdvanceEpochReceipt {
        new_epoch,
        channel_group_next,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EpochCounter;
    use crate::RewardClearingError as E;
    use crate::test_support::*;

    fn channel_group(current_epoch_index: EpochCounter) -> ChannelGroup<Id> {
        ChannelGroup {
            id: CHANNEL_GROUP,
            master_start_time_unix_seconds: START,
            epoch_duration_seconds: DURATION,
            current_epoch_index,
        }
    }

    fn advance(channel_group: ChannelGroup<Id>, now: Seconds) -> Result<AdvanceEpochReceipt<Id>> {
        advance_epoch(AdvanceEpochArgs {
            now,
            channel_group,
            current_epoch: epoch(),
        })
    }

    #[test]
    fn next_epoch_starts_when_the_current_one_ends() {
        let Ok(advanced) = advance(channel_group(3), START + DURATION) else {
            panic!("the current epoch has ended");
        };

        assert_eq!(advanced.new_epoch.epoch_counter, 4);
        assert_eq!(advanced.new_epoch.start_time_unix_seconds, START + DURATION);
        assert_eq!(advanced.new_epoch.duration_seconds, DURATION);
        assert_eq!(advanced.channel_group_next.current_epoch_index, 4);
    }

    #[test]
    fn rejects_before_the_current_epoch_ends() {
        let result = advance(channel_group(3), START + DURATION - 1);
        assert_eq!(result.err(), Some(E::EpochNotEnded));
    }

    #[test]
    fn rejects_epoch_counter_mismatch_with_group() {
        let result = advance(channel_group(2), START + DURATION);
        assert_eq!(result.err(), Some(E::ChannelGroupCounterMismatch));
    }

    #[test]
    fn rejects_epoch_from_another_channel_group() {
        let mut channel_group = channel_group(3);
        channel_group.id = Id(8);

        let result = advance(channel_group, START + DURATION);
        assert_eq!(result.err(), Some(E::EpochChannelGroupMismatch));
    }

    #[test]
    fn rejects_an_unrepresentable_successor_end() {
        use crate::{CreateChannelGroupArgs, MIN_EPOCH_DURATION_SECONDS, create_channel_group};

        let created = create_channel_group(CreateChannelGroupArgs {
            channel_group_id: CHANNEL_GROUP,
            master_start_time_unix_seconds: u64::MAX - MIN_EPOCH_DURATION_SECONDS,
            epoch_duration_seconds: MIN_EPOCH_DURATION_SECONDS,
        })
        .unwrap();
        let result = advance_epoch(AdvanceEpochArgs {
            now: u64::MAX,
            channel_group: created.new_channel_group,
            current_epoch: created.initial_epoch,
        });

        assert_eq!(result.err(), Some(E::EpochEndTimeOverflow));
    }
}
