use mayf_fixed_number::FixedNumber;

use crate::models::Identifiable;
use crate::models::channel::Channel;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::channel_group::ChannelGroup;
use crate::models::epoch::Epoch;
use crate::{EpochTerms, Result, Seconds};

pub struct CreateChannelArgs<I: Identifiable, N: FixedNumber> {
    pub channel_id: I,
    pub reward_id: I,
    pub default_terms: EpochTerms<N>,
    pub channel_group: ChannelGroup<I>,
    pub active_epoch: Epoch<I>,
    pub now: Seconds,
}

pub struct CreateChannelReceipt<I: Identifiable, N: FixedNumber> {
    pub new_channel: Channel<I, N>,
    pub new_channel_epoch: ChannelEpoch<I, N>,
}

/// Create an enabled channel and open its first epoch in the group's live epoch.
/// The host must store both returned models atomically.
pub fn create_channel<N, I>(args: CreateChannelArgs<I, N>) -> Result<CreateChannelReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    args.active_epoch
        .validate_active(&args.channel_group, args.now)?;

    let new_channel_epoch = ChannelEpoch::new(
        args.channel_id,
        args.active_epoch.epoch_counter,
        args.default_terms,
    );
    let new_channel = Channel {
        id: args.channel_id,
        channel_group_id: args.channel_group.id,
        reward_id: args.reward_id,
        default_terms: args.default_terms,
        disabled: false,
        last_created_epoch_counter: args.active_epoch.epoch_counter,
    };

    Ok(CreateChannelReceipt {
        new_channel,
        new_channel_epoch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RewardClearingError as E;
    use crate::test_support::*;
    use crate::{CreateChannelEpochArgs, create_channel_epoch};

    fn args() -> CreateChannelArgs<Id, Number> {
        CreateChannelArgs {
            channel_id: CHANNEL,
            reward_id: Id(5),
            default_terms: terms(1, 2),
            channel_group: channel_group(),
            active_epoch: epoch(),
            now: START,
        }
    }

    #[test]
    fn opens_the_first_epoch_even_at_counter_zero() {
        let mut args = args();
        args.channel_group.current_epoch_index = 0;
        args.channel_group.master_start_time_unix_seconds = START;
        args.active_epoch.epoch_counter = 0;
        let group = args.channel_group.clone();
        let epoch = args.active_epoch.clone();
        let created = create_channel(args).unwrap();

        assert!(!created.new_channel.disabled);
        assert_eq!(created.new_channel.last_created_epoch_counter, 0);
        let channel_epoch = created.new_channel_epoch;
        assert_eq!(channel_epoch.channel_id, created.new_channel.id);
        assert_eq!(channel_epoch.epoch_counter, 0);
        assert_eq!(channel_epoch.terms, created.new_channel.default_terms);
        assert_eq!(channel_epoch.clearing_credits_balance, 0);
        assert_eq!(channel_epoch.reward_pot_balance, 0);
        assert_eq!(channel_epoch.weighted_shares_balance, shares(0));
        assert_eq!(channel_epoch.final_totals, None);

        let duplicate = create_channel_epoch(CreateChannelEpochArgs {
            channel: created.new_channel,
            channel_group: group,
            active_epoch: epoch,
            now: START,
        });
        assert_eq!(duplicate.err(), Some(E::ChannelEpochAlreadyCreated));
    }

    #[test]
    fn requires_the_groups_current_live_epoch() {
        let mut another_group = args();
        another_group.channel_group.id = Id(8);
        let mut stale_epoch = args();
        stale_epoch.channel_group.current_epoch_index += 1;

        for (args, expected) in [
            (
                CreateChannelArgs {
                    now: START - 1,
                    ..args()
                },
                E::EpochNotStarted,
            ),
            (
                CreateChannelArgs {
                    now: START + DURATION,
                    ..args()
                },
                E::EpochEnded,
            ),
            (another_group, E::EpochChannelGroupMismatch),
            (stale_epoch, E::ChannelGroupCounterMismatch),
        ] {
            assert_eq!(create_channel(args).err(), Some(expected));
        }
    }
}
