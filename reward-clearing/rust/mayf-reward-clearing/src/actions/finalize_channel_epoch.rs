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
/// This freezes the reward pot and total shares that every claim uses.
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
    use crate::models::channel_epoch::FinalizedTotals;
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
    fn freezes_the_reward_pot_and_weighted_shares() {
        // 100 credits at the start earn 200 shares. A pot of 400 pays 2 per share.
        let mut channel_epoch = first_deposit(channel_epoch(), START, 100)
            .unwrap()
            .channel_epoch_next;
        channel_epoch.reward_pot_balance = 400;

        let finalized = finalize(channel_epoch, END).unwrap();
        assert_eq!(
            finalized.channel_epoch_next.final_totals,
            Some(FinalizedTotals {
                reward_pot: 400,
                weighted_shares: shares(200),
            })
        );
    }

    #[test]
    fn epoch_without_shares_still_finalizes() {
        let mut channel_epoch = channel_epoch();
        channel_epoch.reward_pot_balance = 400;

        let finalized = finalize(channel_epoch, END).unwrap();
        assert_eq!(
            finalized.channel_epoch_next.final_totals,
            Some(FinalizedTotals {
                reward_pot: 400,
                weighted_shares: shares(0),
            })
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

    #[test]
    fn finalization_closes_all_live_epoch_actions() {
        use crate::{
            AddRewardArgs, CreateBidderPositionArgs, WithdrawCreditsArgs, add_reward,
            create_bidder_position, withdraw_credits,
        };

        for reward in [0, 400] {
            let deposited = first_deposit(channel_epoch(), START, 100).unwrap();
            let mut funded = deposited.channel_epoch_next;
            funded.reward_pot_balance = reward;
            let finalized = finalize(funded, END).unwrap().channel_epoch_next;

            // The saved finalized state is authoritative, even if a host supplies
            // a time that would otherwise permit changes to the epoch.
            for now in [START, END] {
                let errors = [
                    create_bidder_position(CreateBidderPositionArgs {
                        position_id: deposited.bidder_position_next.id,
                        channel_group: channel_group(),
                        channel: channel(),
                        channel_epoch: finalized.clone(),
                        epoch: epoch(),
                        now,
                    })
                    .err(),
                    add_reward(AddRewardArgs {
                        channel_group: channel_group(),
                        epoch: epoch(),
                        channel: channel(),
                        channel_epoch: finalized.clone(),
                        now,
                        amount: 1,
                    })
                    .err(),
                    first_deposit(finalized.clone(), now, 1).err(),
                    withdraw_credits(WithdrawCreditsArgs {
                        channel_group: channel_group(),
                        bidder_position: deposited.bidder_position_next.clone(),
                        channel: channel(),
                        channel_epoch: finalized.clone(),
                        epoch: epoch(),
                        now,
                        credits_to_withdraw: 1,
                    })
                    .err(),
                ];
                assert_eq!(errors, [Some(E::ChannelEpochAlreadyFinalized); 4]);
            }
        }
    }
}
