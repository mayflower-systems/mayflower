use mayf_fixed_number::{FixedNumber, IntegralRounding, NonNeg};

use crate::Amount;
use crate::Result;
use crate::RewardClearingError;
use crate::Seconds;
use crate::models::Identifiable;
use crate::models::bidder_position::BidderPosition;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::epoch::Epoch;

pub struct WithdrawCreditsArgs<I: Identifiable, N: FixedNumber> {
    pub bidder_position: BidderPosition<I, N>,
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

    /// Credits lost to the exit penalty.
    /// They leave the channel epoch's balance, so the host must burn them.
    pub credits_forfeited: Amount,
}

/// Withdraw credits from the bidder's position, newest lots first.
/// The withdrawn credits' shares are destroyed, and the exit penalty grows through the epoch.
pub fn withdraw_credits<N, I>(
    args: WithdrawCreditsArgs<I, N>,
) -> Result<WithdrawCreditsReceipt<I, N>>
where
    I: Identifiable,
    N: IntegralRounding,
{
    args.epoch.validate_active(args.now)?;
    args.channel_epoch.validate_epoch(&args.epoch)?;
    args.bidder_position
        .validate_channel_epoch(&args.channel_epoch)?;

    let (bidder_position_next, shares_removed) = args
        .bidder_position
        .withdraw_credits(args.credits_to_withdraw)?;

    let exit_penalty = args.channel_epoch.exit_penalty(&args.epoch, args.now)?;
    let credits_returned = credits_after_penalty(args.credits_to_withdraw, exit_penalty)?;
    let credits_forfeited = args
        .credits_to_withdraw
        .checked_sub(credits_returned)
        .ok_or(RewardClearingError::CreditsForfeitedUnderflow)?;

    let channel_epoch_next = args
        .channel_epoch
        .withdraw_credits(args.credits_to_withdraw, shares_removed)?;

    Ok(WithdrawCreditsReceipt {
        bidder_position_next,
        channel_epoch_next,
        credits_returned,
        credits_forfeited,
    })
}

/// Credits left after the penalty, rounded down to whole credits:
///
///   returned = floor(credits * (1 - penalty))
fn credits_after_penalty<N: IntegralRounding>(
    credits: Amount,
    penalty: NonNeg<N>,
) -> Result<Amount> {
    NonNeg::<N>::one()
        .checked_sub(penalty)
        .and_then(|kept_fraction| NonNeg::<N>::from_u64(credits)?.checked_mul_floor(kept_fraction))
        .and_then(|kept| kept.get().checked_floor_to_u64())
        .map_err(|_| RewardClearingError::CreditsReturnedOverflow)
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
            bidder_position: from.bidder_position_next,
            channel_epoch: from.channel_epoch_next,
            epoch: epoch(),
            now,
            credits_to_withdraw,
        })
    }

    #[test]
    fn exit_penalty_rises_with_the_exponent() {
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
            100,
        )
        .unwrap();

        // Pops the newest lot (100 credits, 150 shares), then takes 50 credits
        // and 100 shares from the oldest lot (100 credits, 200 shares).
        let withdrawn = withdraw(second, HALFWAY, 150).unwrap();

        // floor(150 * 0.75) = 112
        assert_eq!(withdrawn.credits_returned, 112);
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
            bidder_position: position,
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
    fn rejects_withdrawal_after_epoch_end() {
        let deposited = first_deposit(channel_epoch(), START, 100).unwrap();
        let result = withdraw(deposited, START + DURATION, 100);
        assert_eq!(result.err(), Some(E::EpochEnded));
    }
}
