use mayf_fixed_number::{FixedNumber, IntegralRounding};

use crate::Amount;
use crate::Result;
use crate::models::Identifiable;
use crate::models::bidder_position::BidderPosition;
use crate::models::channel_epoch::ChannelEpoch;

pub struct ClaimArgs<I: Identifiable, N: FixedNumber> {
    pub bidder_position: BidderPosition<I, N>,
    pub channel_epoch: ChannelEpoch<I, N>,
}

pub struct ClaimReceipt<I: Identifiable, N: FixedNumber> {
    pub bidder_position_next: BidderPosition<I, N>,
    pub channel_epoch_next: ChannelEpoch<I, N>,

    /// Reward the host pays the bidder from the pot
    pub reward_earned: Amount,
}

/// Claim a bidder's part of a finalized channel epoch's reward pot.
/// The claim freezes the position, so it can never deposit, withdraw, or claim again.
/// A claim that rounds down to zero reward still retires its shares.
pub fn claim<N, I>(args: ClaimArgs<I, N>) -> Result<ClaimReceipt<I, N>>
where
    I: Identifiable,
    N: IntegralRounding,
{
    args.bidder_position
        .validate_channel_epoch(&args.channel_epoch)?;

    let (bidder_position_next, shares_claimed) = args.bidder_position.claim()?;

    let (channel_epoch_next, reward_earned) = args.channel_epoch.claim(shares_claimed)?;

    Ok(ClaimReceipt {
        bidder_position_next,
        channel_epoch_next,
        reward_earned,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RewardClearingError as E;
    use crate::models::bidder_position::BidderLot;
    use crate::test_support::*;

    const ALICE: Id = Id(10);
    const BOB: Id = Id(11);

    /// A channel epoch finalized with a pot of `reward` over 290 shares
    fn finalized(reward: Amount) -> ChannelEpoch<Id, Number> {
        let mut channel_epoch = channel_epoch();
        channel_epoch.reward_pot_balance = reward;
        channel_epoch.weighted_shares_balance = shares(290);
        channel_epoch.finalize().unwrap()
    }

    /// A position with one lot
    fn position(
        id: Id,
        whole_shares: u64,
        channel_epoch: &ChannelEpoch<Id, Number>,
    ) -> BidderPosition<Id, Number> {
        BidderPosition {
            id,
            channel_epoch_id: channel_epoch.id(),
            lots: vec![BidderLot {
                bidder_position_id: id,
                credits: 100,
                shares: shares(whole_shares),
            }],
            claimed: false,
        }
    }

    fn claim_by(
        bidder_position: BidderPosition<Id, Number>,
        channel_epoch: ChannelEpoch<Id, Number>,
    ) -> Result<ClaimReceipt<Id, Number>> {
        claim(ClaimArgs {
            bidder_position,
            channel_epoch,
        })
    }

    /// The claims in the wiki's "Revenue Clearing Time Weight" example
    #[test]
    fn matches_wiki_worked_example() {
        let channel_epoch = finalized(90);
        let alice = position(ALICE, 200, &channel_epoch);
        let bob = position(BOB, 90, &channel_epoch);

        let alice_claim = claim_by(alice, channel_epoch).unwrap();
        let bob_claim = claim_by(bob, alice_claim.channel_epoch_next).unwrap();

        // floor(200 * 90 / 290) = 62 and floor(90 * 90 / 290) = 27
        assert_eq!(alice_claim.reward_earned, 62);
        assert_eq!(bob_claim.reward_earned, 27);
        assert!(alice_claim.bidder_position_next.claimed);

        // One reward unit stays in the pot for the protocol, and every share has claimed.
        let channel_epoch = &bob_claim.channel_epoch_next;
        assert_eq!(channel_epoch.reward_pot_balance, 1);
        assert_eq!(channel_epoch.weighted_shares_balance, shares(0));
    }

    #[test]
    fn claim_order_does_not_change_payouts() {
        let channel_epoch = finalized(90);
        let alice = position(ALICE, 200, &channel_epoch);
        let bob = position(BOB, 90, &channel_epoch);

        let bob_claim = claim_by(bob, channel_epoch).unwrap();
        let alice_claim = claim_by(alice, bob_claim.channel_epoch_next).unwrap();

        assert_eq!(bob_claim.reward_earned, 27);
        assert_eq!(alice_claim.reward_earned, 62);
    }

    #[test]
    fn zero_reward_claim_still_retires_shares() {
        // floor(90 * 1 / 290) = 0
        let channel_epoch = finalized(1);
        let bob = position(BOB, 90, &channel_epoch);

        let bob_claim = claim_by(bob, channel_epoch).unwrap();

        assert_eq!(bob_claim.reward_earned, 0);
        assert!(bob_claim.bidder_position_next.claimed);
        let channel_epoch = &bob_claim.channel_epoch_next;
        assert_eq!(channel_epoch.weighted_shares_balance, shares(200));
    }

    #[test]
    fn rejects_second_claim() {
        let channel_epoch = finalized(90);
        let alice = position(ALICE, 200, &channel_epoch);

        let first = claim_by(alice, channel_epoch).unwrap();
        let second = claim_by(first.bidder_position_next, first.channel_epoch_next);

        assert_eq!(second.err(), Some(E::BidderPositionClaimed));
    }

    #[test]
    fn rejects_claim_before_finalize() {
        let channel_epoch = channel_epoch();
        let alice = position(ALICE, 200, &channel_epoch);

        let result = claim_by(alice, channel_epoch);
        assert_eq!(result.err(), Some(E::ChannelEpochNotFinalized));
    }

    #[test]
    fn rejects_position_from_another_channel_epoch() {
        let channel_epoch = finalized(90);
        let mut alice = position(ALICE, 200, &channel_epoch);
        alice.channel_epoch_id.epoch_counter = 4;

        let result = claim_by(alice, channel_epoch);
        assert_eq!(result.err(), Some(E::BidderPositionChannelEpochMismatch));
    }
}
