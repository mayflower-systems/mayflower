use mayf_fixed_number::FixedNumber;

use crate::Amount;
use crate::Result;
use crate::Seconds;
use crate::models::Identifiable;
use crate::models::bidder_position::BidderPosition;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::epoch::Epoch;

pub struct DepositCreditsArgs<I: Identifiable, N: FixedNumber> {
    pub bidder_position: BidderPosition<I, N>,
    pub channel_epoch: ChannelEpoch<I, N>,
    pub epoch: Epoch<I>,
    pub now: Seconds,
    pub credits_to_deposit: Amount,
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
    args.epoch.validate_active(args.now)?;
    args.channel_epoch.validate_epoch(&args.epoch)?;
    args.bidder_position
        .validate_channel_epoch(&args.channel_epoch)?;

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
    fn rejects_position_from_another_channel_epoch() {
        let channel_epoch = channel_epoch();
        let mut position = empty_position(&channel_epoch);
        position.channel_epoch_id = ChannelEpochId {
            channel_id: Id(9),
            epoch_counter: 3,
        };

        let result = deposit(position, channel_epoch, START, 100);
        assert_eq!(result.err(), Some(E::BidderPositionChannelEpochMismatch));
    }
}
