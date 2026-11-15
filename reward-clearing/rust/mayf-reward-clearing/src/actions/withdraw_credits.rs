use mayf_fixed_number::{FixedNumber, IntegralRounding};

use crate::Amount;
use crate::Result;
use crate::Seconds;
use crate::models::Identifiable;
use crate::models::bidder_position::BidderPosition;
use crate::models::channel::Channel;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::channel_group::ChannelGroup;
use crate::models::epoch::Epoch;

pub struct WithdrawCreditsArgs<I: Identifiable, N: FixedNumber> {
    pub channel_group: ChannelGroup<I>,
    pub bidder_position: BidderPosition<I, N>,
    pub channel: Channel<I, N>,
    pub channel_epoch: ChannelEpoch<I, N>,
    pub epoch: Epoch<I>,
    pub now: Seconds,
    pub credits_to_withdraw: Amount,
}

pub struct WithdrawCreditsReceipt<I: Identifiable, N: FixedNumber> {
    pub bidder_position_next: BidderPosition<I, N>,
    pub channel_epoch_next: ChannelEpoch<I, N>,

    /// Credits the host returns to the bidder
    pub credits_returned: Amount,

    /// Credits the host burns from escrow.
    pub credits_forfeited: Amount,
}

/// Withdraw credits in the live epoch, consuming the newest lots first.
///
/// The action removes the associated shares and computes one refund for the
/// full requested amount at the current exit penalty, even across several
/// lots. The host refunds `credits_returned`, burns `credits_forfeited`, and
/// persists both updated models together.
pub fn withdraw_credits<N, I>(
    args: WithdrawCreditsArgs<I, N>,
) -> Result<WithdrawCreditsReceipt<I, N>>
where
    I: Identifiable,
    N: IntegralRounding,
{
    args.channel_epoch.validate_unfinalized()?;
    args.epoch.validate_active(&args.channel_group, args.now)?;
    args.channel_epoch
        .validate_channel_and_epoch(&args.channel, &args.epoch)?;
    args.bidder_position
        .validate_channel_epoch(&args.channel_epoch)?;

    let (bidder_position_next, shares_removed) = args
        .bidder_position
        .withdraw_credits(args.credits_to_withdraw)?;

    let withdrawal = args
        .channel_epoch
        .auction_window(&args.epoch)
        .split_withdrawal(args.now, args.credits_to_withdraw)?;

    let channel_epoch_next = args
        .channel_epoch
        .withdraw_credits(args.credits_to_withdraw, shares_removed)?;

    Ok(WithdrawCreditsReceipt {
        bidder_position_next,
        channel_epoch_next,
        credits_returned: withdrawal.returned,
        credits_forfeited: withdrawal.forfeited,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DepositCreditsReceipt;
    use crate::RewardClearingError as E;
    use crate::models::bidder_position::BidderLot;
    use crate::test_support::*;

    fn withdraw(
        from: DepositCreditsReceipt<Id, Number>,
        now: Seconds,
        credits_to_withdraw: Amount,
    ) -> Result<WithdrawCreditsReceipt<Id, Number>> {
        withdraw_credits(WithdrawCreditsArgs {
            channel_group: channel_group(),
            bidder_position: from.bidder_position_next,
            channel: channel(),
            channel_epoch: from.channel_epoch_next,
            epoch: epoch(),
            now,
            credits_to_withdraw,
        })
    }

    #[test]
    fn exit_penalty_rises_with_time_and_falls_with_higher_exponents() {
        // (exit penalty exponent, seconds elapsed, credits returned from 100)
        let cases = [
            (1, 50, 50),
            (2, 50, 75),
            (4, 50, 93),
            // 0.9 is not on the binary grid. The penalty rounds up, so 9 credits return, not 10.
            (1, 90, 9),
        ];

        for (exponent, elapsed, expected) in cases {
            let deposited = first_deposit(channel_epoch_with_terms(1, exponent), START, 100);
            let withdrawn = withdraw(deposited.unwrap(), START + elapsed, 100).unwrap();
            assert_eq!(withdrawn.credits_returned, expected);
            assert_eq!(withdrawn.credits_forfeited, 100 - expected);
        }
    }

    #[test]
    fn full_withdrawal_at_start_returns_everything() {
        let deposited = first_deposit(channel_epoch(), START, 100).unwrap();
        let withdrawn = withdraw(deposited, START, 100).unwrap();

        assert_eq!(withdrawn.credits_returned, 100);
        assert!(withdrawn.bidder_position_next.lots.is_empty());

        let channel_epoch = &withdrawn.channel_epoch_next;
        assert_eq!(channel_epoch.clearing_credits_balance, 0);
        assert_eq!(channel_epoch.weighted_shares_balance, shares(0));
    }

    #[test]
    fn indivisible_withdrawal_forfeits_the_whole_credit() {
        let deposited = first_deposit(channel_epoch(), START, 100).unwrap();
        let withdrawn = withdraw(deposited, START + 1, 1).unwrap();

        assert_eq!(withdrawn.credits_returned, 0);
        assert_eq!(withdrawn.credits_forfeited, 1);
    }

    /// Bob's withdrawal in the wiki's "Revenue Clearing Time Weight" example
    #[test]
    fn matches_wiki_worked_example() {
        let deposited = first_deposit(channel_epoch(), HALFWAY, 100).unwrap();
        let withdrawn = withdraw(deposited, HALFWAY, 40).unwrap();

        assert_eq!(withdrawn.credits_returned, 30);
        assert_eq!(withdrawn.credits_forfeited, 10);

        let lots = &withdrawn.bidder_position_next.lots;
        assert_eq!(lots.len(), 1);
        assert_eq!((lots[0].credits, lots[0].shares), (60, shares(90)));

        let channel_epoch = &withdrawn.channel_epoch_next;
        assert_eq!(channel_epoch.clearing_credits_balance, 60);
        assert_eq!(channel_epoch.weighted_shares_balance, shares(90));
    }

    #[test]
    fn spans_lots_newest_first() {
        let first = first_deposit(channel_epoch(), START, 100).unwrap();
        let second = deposit(
            first.bidder_position_next,
            first.channel_epoch_next,
            HALFWAY,
            102,
        )
        .unwrap();

        // Pops the newest lot (102 credits, 153 shares), then takes 50 credits
        // and 100 shares from the oldest lot (100 credits, 200 shares).
        let withdrawn = withdraw(second, HALFWAY, 152).unwrap();

        // Round the combined refund of 114 once. Rounding the lots'
        // refunds of 76.5 and 37.5 separately would return only 113.
        assert_eq!(withdrawn.credits_returned, 114);
        assert_eq!(withdrawn.credits_forfeited, 38);

        let lots = &withdrawn.bidder_position_next.lots;
        assert_eq!(lots.len(), 1);
        assert_eq!((lots[0].credits, lots[0].shares), (50, shares(100)));

        let channel_epoch = &withdrawn.channel_epoch_next;
        assert_eq!(channel_epoch.clearing_credits_balance, 50);
        assert_eq!(channel_epoch.weighted_shares_balance, shares(100));
    }

    #[test]
    fn partial_withdrawal_rounds_shares_removed_up() {
        // 1 of the lot's 3 credits owns 4/3 shares, which is not on the binary grid.
        let mut channel_epoch = channel_epoch();
        channel_epoch.clearing_credits_balance = 3;
        channel_epoch.weighted_shares_balance = shares(4);

        let mut position = empty_position(&channel_epoch);
        position.lots.push(BidderLot {
            bidder_position_id: BIDDER,
            credits: 3,
            shares: shares(4),
        });

        let withdrawn = withdraw_credits(WithdrawCreditsArgs {
            channel_group: channel_group(),
            bidder_position: position,
            channel: channel(),
            channel_epoch,
            epoch: epoch(),
            now: START,
            credits_to_withdraw: 1,
        })
        .unwrap();

        // More than 4/3 shares removed leaves fewer than 8/3 behind.
        let remaining = withdrawn.bidder_position_next.lots[0].shares;
        assert!(remaining.checked_mul_floor(shares(3)).unwrap() < shares(8));
    }

    #[test]
    fn rejects_bad_amounts() {
        let cases = [
            (0, E::WithdrawalTooSmall),
            (101, E::WithdrawalExceedsPositionCredits),
        ];

        for (credits, error) in cases {
            let deposited = first_deposit(channel_epoch(), START, 100).unwrap();
            let result = withdraw(deposited, START, credits);
            assert_eq!(result.err(), Some(error));
        }
    }

    #[test]
    fn rejects_claimed_position() {
        let mut deposited = first_deposit(channel_epoch(), START, 100).unwrap();
        deposited.bidder_position_next.claimed = true;

        let result = withdraw(deposited, START, 50);
        assert_eq!(result.err(), Some(E::BidderPositionClaimed));
    }

    #[test]
    fn rejects_lot_from_another_position() {
        let mut deposited = first_deposit(channel_epoch(), START, 100).unwrap();
        deposited.bidder_position_next.lots[0].bidder_position_id = Id(9);

        let result = withdraw(deposited, START, 50);
        assert_eq!(result.err(), Some(E::BidderLotPositionMismatch));
    }

    #[test]
    fn rejects_withdrawal_after_epoch_end() {
        let deposited = first_deposit(channel_epoch(), START, 100).unwrap();
        let result = withdraw(deposited, START + DURATION, 100);
        assert_eq!(result.err(), Some(E::EpochEnded));
    }
}
