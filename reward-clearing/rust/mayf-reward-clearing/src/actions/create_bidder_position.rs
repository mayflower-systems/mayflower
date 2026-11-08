use mayf_fixed_number::FixedNumber;

use crate::models::Identifiable;
use crate::models::bidder_position::BidderPosition;
use crate::models::channel::Channel;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::channel_group::ChannelGroup;
use crate::models::epoch::Epoch;
use crate::{Result, Seconds};

pub struct CreateBidderPositionArgs<I: Identifiable, N: FixedNumber> {
    pub position_id: I,
    pub channel_group: ChannelGroup<I>,
    pub channel: Channel<I, N>,
    pub channel_epoch: ChannelEpoch<I, N>,
    pub epoch: Epoch<I>,
    pub now: Seconds,
}

pub struct CreateBidderPositionReceipt<I: Identifiable, N: FixedNumber> {
    pub bidder_position: BidderPosition<I, N>,
}

/// Open an empty position in an unfinalized channel epoch during its group's live epoch.
pub fn create_bidder_position<N, I>(
    args: CreateBidderPositionArgs<I, N>,
) -> Result<CreateBidderPositionReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    args.channel_epoch.validate_unfinalized()?;
    args.epoch.validate_active(&args.channel_group, args.now)?;
    args.channel_epoch
        .validate_channel_and_epoch(&args.channel, &args.epoch)?;

    Ok(CreateBidderPositionReceipt {
        bidder_position: BidderPosition::new(args.position_id, args.channel_epoch.id()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RewardClearingError as E;
    use crate::test_support::*;

    fn args() -> CreateBidderPositionArgs<Id, Number> {
        CreateBidderPositionArgs {
            position_id: BIDDER,
            channel_group: channel_group(),
            channel: channel(),
            channel_epoch: channel_epoch(),
            epoch: epoch(),
            now: START,
        }
    }

    #[test]
    fn requires_both_the_current_epoch_and_its_live_interval() {
        let mut stale = args();
        stale.channel_group.current_epoch_index += 1;
        let mut another_group = args();
        another_group.channel_group.id = Id(8);
        let mut overflowing_end = args();
        overflowing_end.epoch.start_time_unix_seconds = u64::MAX;
        overflowing_end.epoch.duration_seconds = 1;
        overflowing_end.now = u64::MAX;

        for (args, expected) in [
            (
                CreateBidderPositionArgs {
                    now: START - 1,
                    ..args()
                },
                Some(E::EpochNotStarted),
            ),
            (args(), None),
            (
                CreateBidderPositionArgs {
                    now: START + DURATION - 1,
                    ..args()
                },
                None,
            ),
            // The group counter has not advanced, but the epoch has still ended.
            (
                CreateBidderPositionArgs {
                    now: START + DURATION,
                    ..args()
                },
                Some(E::EpochEnded),
            ),
            // The clock is inside the interval, but the group's counter has moved on.
            (stale, Some(E::ChannelGroupCounterMismatch)),
            (another_group, Some(E::EpochChannelGroupMismatch)),
            (overflowing_end, Some(E::EpochEndTimeOverflow)),
        ] {
            assert_eq!(create_bidder_position(args).err(), expected);
        }
    }

    #[test]
    fn rejects_models_from_different_channels_or_epochs() {
        let mut another_group = args();
        another_group.channel.channel_group_id = Id(8);
        let mut another_channel = args();
        another_channel.channel_epoch.channel_id = Id(9);
        let mut another_epoch = args();
        another_epoch.channel_epoch.epoch_counter += 1;

        for (args, expected) in [
            (another_group, E::ChannelGroupMismatch),
            (another_channel, E::ChannelEpochChannelMismatch),
            (another_epoch, E::ChannelEpochCounterMismatch),
        ] {
            assert_eq!(create_bidder_position(args).err(), Some(expected));
        }
    }
}
