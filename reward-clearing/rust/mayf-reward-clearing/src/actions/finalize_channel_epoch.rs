use mayf_fixed_number::FixedNumber;

use crate::Result;
use crate::RewardClearingError;
use crate::Seconds;
use crate::models::Identifiable;
use crate::models::channel::Channel;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::epoch::Epoch;

pub struct FinalizeChannelEpochArgs<I: Identifiable, N: FixedNumber> {
    pub channel: Channel<I, N>,
    pub channel_epoch: ChannelEpoch<I, N>,
    pub epoch: Epoch<I>,
    pub now: Seconds,
}

pub struct FinalizeChannelEpochReceipt<I: Identifiable, N: FixedNumber> {
    pub channel_epoch_next: ChannelEpoch<I, N>,
}

/// Finalize a channel epoch once its epoch has ended.
/// This fixes the reward per weighted share that every claim uses.
/// A channel epoch can be finalized only once.
/// The host burns the remaining clearing credits.
pub fn finalize_channel_epoch<N, I>(
    args: FinalizeChannelEpochArgs<I, N>,
) -> Result<FinalizeChannelEpochReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    if !args.epoch.is_finished(args.now)? {
        return Err(RewardClearingError::EpochNotEnded);
    }

    args.channel_epoch
        .validate_channel_and_epoch(&args.channel, &args.epoch)?;

    let channel_epoch_next = args.channel_epoch.finalize()?;

    Ok(FinalizeChannelEpochReceipt { channel_epoch_next })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RewardClearingError as E;
    use crate::test_support::*;

    const END: Seconds = START + DURATION;

    fn finalize(
        channel_epoch: ChannelEpoch<Id, Number>,
        now: Seconds,
    ) -> Result<FinalizeChannelEpochReceipt<Id, Number>> {
        finalize_channel_epoch(FinalizeChannelEpochArgs {
            channel: channel(),
            channel_epoch,
            epoch: epoch(),
            now,
        })
    }

    #[test]
    fn fixes_the_reward_per_share() {
        // 100 credits at the start earn 200 shares. A pot of 400 pays 2 per share.
        let mut channel_epoch = first_deposit(channel_epoch(), START, 100)
            .unwrap()
            .channel_epoch_next;
        channel_epoch.reward_pot_balance = 400;

        let finalized = finalize(channel_epoch, END).unwrap();
        assert_eq!(
            finalized.channel_epoch_next.final_share_rate,
            Some(shares(2))
        );
    }

    #[test]
    fn epoch_without_shares_finalizes_at_zero() {
        let mut channel_epoch = channel_epoch();
        channel_epoch.reward_pot_balance = 400;

        let finalized = finalize(channel_epoch, END).unwrap();
        assert_eq!(
            finalized.channel_epoch_next.final_share_rate,
            Some(shares(0))
        );
    }

    #[test]
    fn rejects_second_finalize() {
        let finalized = finalize(channel_epoch(), END).unwrap();
        let result = finalize(finalized.channel_epoch_next, END);
        assert_eq!(result.err(), Some(E::ChannelEpochAlreadyFinalized));
    }

    #[test]
    fn rejects_finalize_before_epoch_end() {
        let result = finalize(channel_epoch(), END - 1);
        assert_eq!(result.err(), Some(E::EpochNotEnded));
    }
}
