use crate::{
    Seconds,
    models::{Identifiable, channel_group::ChannelGroup, epoch::Epoch},
};

pub struct AdvanceEpochArgs<I: Identifiable> {
    pub now_unix_seconds: Seconds,
    pub channel_group: ChannelGroup<I>,
    pub current_epoch: Epoch<I>,
}

pub struct AdvanceEpochReceipt<I: Identifiable> {
    pub new_epoch: Epoch<I>,
    pub channel_group_next: ChannelGroup<I>,
}

pub enum AdvanceEpochError {
    CurrentEpochNotFinished,
    CurrentEpochCounterMismatchWithGroup,
}

pub fn advance_epoch<I>(
    args: AdvanceEpochArgs<I>,
) -> Result<AdvanceEpochReceipt<I>, AdvanceEpochError>
where
    I: Identifiable,
{
    if !args.current_epoch.is_finished(args.now_unix_seconds) {
        return Err(AdvanceEpochError::CurrentEpochNotFinished);
    }

    if args.current_epoch.epoch_counter != args.channel_group.current_epoch_index {
        return Err(AdvanceEpochError::CurrentEpochCounterMismatchWithGroup);
    }

    // the new epoch starts at the end time of the current epoch
    let start_time_unix_seconds = args.current_epoch.end_time_unix_seconds();

    let next_epoch_counter = args
        .channel_group
        .current_epoch_index
        .checked_add(1)
        .unwrap();

    let new_epoch = Epoch::new(
        args.channel_group.id,
        next_epoch_counter,
        start_time_unix_seconds,
        args.channel_group.epoch_duration_seconds,
    );

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
