use mayf_fixed_number::FixedNumber;

use crate::Amount;
use crate::Result;
use crate::RewardClearingError;
use crate::Seconds;
use crate::models::Identifiable;
use crate::models::bidder_position::BidderPosition;
use crate::models::channel::Channel;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::channel_group::ChannelGroup;
use crate::models::epoch::Epoch;

pub struct DepositCreditsArgs<I: Identifiable, N: FixedNumber> {
    pub channel_group: ChannelGroup<I>,
    pub bidder_position: BidderPosition<I, N>,
    pub channel: Channel<I, N>,
    pub channel_epoch: ChannelEpoch<I, N>,
    pub epoch: Epoch<I>,
    pub now: Seconds,
    pub credits_to_deposit: Amount,
    /// Maximum live lots permitted by the host's storage policy.
    pub max_lot_count: u32,
}

pub struct DepositCreditsReceipt<I: Identifiable, N: FixedNumber> {
    pub bidder_position_next: BidderPosition<I, N>,
    pub channel_epoch_next: ChannelEpoch<I, N>,
}

/// Deposit credits as a new lot on the bidder's position.
/// Earlier deposits get a higher entry weight, so they earn more weighted shares per credit.
pub fn deposit_credits<N, I>(args: DepositCreditsArgs<I, N>) -> Result<DepositCreditsReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    args.channel_epoch.validate_unfinalized()?;
    args.epoch.validate_active(&args.channel_group, args.now)?;
    args.channel_epoch
        .validate_channel_and_epoch(&args.channel, &args.epoch)?;
    args.bidder_position
        .validate_channel_epoch(&args.channel_epoch)?;

    if args.bidder_position.lots.len() >= args.max_lot_count as usize {
        return Err(RewardClearingError::LotLimitReached);
    }

    let entry_weight = args.channel_epoch.entry_weight(&args.epoch, args.now)?;

    let (bidder_position_next, shares_issued) = args
        .bidder_position
        .deposit_credits(args.credits_to_deposit, entry_weight)?;

    let channel_epoch_next = args
        .channel_epoch
        .deposit_credits(args.credits_to_deposit, shares_issued)?;

    Ok(DepositCreditsReceipt {
        bidder_position_next,
        channel_epoch_next,
    })
}

#[cfg(test)]
mod tests {
    use crate::RewardClearingError as E;
    use crate::models::channel_epoch::ChannelEpochId;
    use crate::test_support::*;
    use crate::{WithdrawCreditsArgs, withdraw_credits};

    #[test]
    fn entry_weight_falls_from_one_plus_slope_to_one() {
        // (ramp bonus slope, seconds elapsed, shares for 100 credits)
        let cases = [
            (0, 0, 100),
            (1, 0, 200),
            (1, 50, 150),
            (1, 75, 125),
            (2, 0, 300),
            (2, 50, 200),
        ];

        for (slope, elapsed, expected) in cases {
            let channel_epoch = channel_epoch_with_terms(slope, 2);
            let deposited = first_deposit(channel_epoch, START + elapsed, 100).unwrap();
            assert_eq!(
                deposited.bidder_position_next.lots[0].shares,
                shares(expected)
            );
        }
    }

    #[test]
    fn rounds_shares_down() {
        // The exact weight 1.99 is not on the binary grid, so 100 credits earn just under 199 shares.
        let deposited = first_deposit(channel_epoch(), START + 1, 100).unwrap();
        let lot_shares = deposited.bidder_position_next.lots[0].shares;
        assert!(lot_shares < shares(199));
        assert!(lot_shares > shares(198));
    }

    #[test]
    fn each_deposit_pushes_a_lot() {
        let first = first_deposit(channel_epoch(), START, 100).unwrap();
        let second = deposit(
            first.bidder_position_next,
            first.channel_epoch_next,
            HALFWAY,
            100,
        )
        .unwrap();

        let lots = &second.bidder_position_next.lots;
        assert_eq!(lots.len(), 2);
        assert!(lots.iter().all(|lot| lot.bidder_position_id == BIDDER));
        assert_eq!((lots[0].credits, lots[0].shares), (100, shares(200)));
        assert_eq!((lots[1].credits, lots[1].shares), (100, shares(150)));

        let channel_epoch = &second.channel_epoch_next;
        assert_eq!(channel_epoch.clearing_credits_balance, 200);
        assert_eq!(channel_epoch.weighted_shares_balance, shares(350));
    }

    #[test]
    fn withdrawing_a_lot_makes_room_for_another_deposit() {
        let mut deposited = first_deposit(channel_epoch(), START, 10).unwrap();
        for _ in 1..8 {
            deposited = deposit(
                deposited.bidder_position_next,
                deposited.channel_epoch_next,
                START,
                10,
            )
            .unwrap();
        }

        let extra = deposit(
            deposited.bidder_position_next.clone(),
            deposited.channel_epoch_next.clone(),
            START,
            10,
        );
        assert_eq!(extra.err(), Some(E::LotLimitReached));

        let withdrawn = withdraw_credits(WithdrawCreditsArgs {
            channel_group: channel_group(),
            bidder_position: deposited.bidder_position_next,
            channel: channel(),
            channel_epoch: deposited.channel_epoch_next,
            epoch: epoch(),
            now: START,
            credits_to_withdraw: 10,
        })
        .unwrap();
        let replenished = deposit(
            withdrawn.bidder_position_next,
            withdrawn.channel_epoch_next,
            START,
            10,
        )
        .unwrap();
        assert_eq!(replenished.bidder_position_next.lots.len(), 8);
    }

    #[test]
    fn rejects_deposit_outside_live_interval() {
        let cases = [
            (START - 1, E::EpochNotStarted),
            (START + DURATION, E::EpochEnded),
        ];

        for (now, error) in cases {
            let result = first_deposit(channel_epoch(), now, 100);
            assert_eq!(result.err(), Some(error));
        }
    }

    #[test]
    fn rejects_zero_credits() {
        let result = first_deposit(channel_epoch(), START, 0);
        assert_eq!(result.err(), Some(E::DepositTooSmall));
    }

    #[test]
    fn rejects_claimed_position() {
        let channel_epoch = channel_epoch();
        let mut position = empty_position(&channel_epoch);
        position.claimed = true;

        let result = deposit(position, channel_epoch, START, 100);
        assert_eq!(result.err(), Some(E::BidderPositionClaimed));
    }

    #[test]
    fn rejects_position_from_another_channel_epoch() {
        let channel_epoch = channel_epoch();
        let mismatches = [
            ChannelEpochId {
                channel_id: Id(9),
                ..channel_epoch.id()
            },
            ChannelEpochId {
                epoch_counter: channel_epoch.epoch_counter + 1,
                ..channel_epoch.id()
            },
        ];
        for channel_epoch_id in mismatches {
            let mut position = empty_position(&channel_epoch);
            position.channel_epoch_id = channel_epoch_id;

            let result = deposit(position, channel_epoch.clone(), START, 100);
            assert_eq!(result.err(), Some(E::BidderPositionChannelEpochMismatch));
        }
    }
}
