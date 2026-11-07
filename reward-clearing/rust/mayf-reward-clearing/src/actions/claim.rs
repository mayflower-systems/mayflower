use mayf_fixed_number::{FixedNumber, IntegralRounding};

use crate::Amount;
use crate::Result;
use crate::models::Identifiable;
use crate::models::bidder_position::BidderPosition;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::channel_epoch_final::ChannelEpochFinal;

pub struct ClaimArgs<I: Identifiable, N: FixedNumber> {
    pub bidder_position: BidderPosition<I, N>,
    pub channel_epoch: ChannelEpoch<I, N>,
    pub channel_epoch_final: ChannelEpochFinal<I, N>,
}

pub struct ClaimReceipt<I: Identifiable, N: FixedNumber> {
    pub bidder_position_next: BidderPosition<I, N>,
    pub channel_epoch_next: ChannelEpoch<I, N>,
    pub channel_epoch_final_next: ChannelEpochFinal<I, N>,

    /// Reward the host pays the bidder from the pot
    pub reward_earned: Amount,
}

/// Claim a bidder's part of a finalized channel epoch's reward pot.
/// The claim retires all of the position's shares, so the position cannot claim twice.
/// A claim that rounds down to zero reward still retires its shares.
pub fn claim<N, I>(args: ClaimArgs<I, N>) -> Result<ClaimReceipt<I, N>>
where
    I: Identifiable,
    N: IntegralRounding,
{
    args.channel_epoch_final
        .validate_channel_epoch(&args.channel_epoch)?;
    args.bidder_position
        .validate_channel_epoch(&args.channel_epoch)?;

    let (bidder_position_next, shares_claimed) = args.bidder_position.claim()?;

    let reward_earned = args.channel_epoch.claim_reward(shares_claimed)?;

    let channel_epoch_final_next = args.channel_epoch_final.claim_shares(shares_claimed)?;

    Ok(ClaimReceipt {
        bidder_position_next,
        channel_epoch_next: args.channel_epoch,
        channel_epoch_final_next,
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

    /// A finalized channel epoch with a pot of `reward` over 290 shares
    fn finalized(reward: Amount) -> (ChannelEpoch<Id, Number>, ChannelEpochFinal<Id, Number>) {
        let mut channel_epoch = channel_epoch();
        channel_epoch.reward_pot_balance = reward;
        channel_epoch.weighted_shares_balance = shares(290);

        let channel_epoch_final = ChannelEpochFinal {
            channel_epoch_id: channel_epoch.id(),
            unclaimed_weighted_shares: shares(290),
        };

        (channel_epoch, channel_epoch_final)
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
        }
    }

    fn claim_by(
        bidder_position: BidderPosition<Id, Number>,
        channel_epoch: ChannelEpoch<Id, Number>,
        channel_epoch_final: ChannelEpochFinal<Id, Number>,
    ) -> Result<ClaimReceipt<Id, Number>> {
        claim(ClaimArgs {
            bidder_position,
            channel_epoch,
            channel_epoch_final,
        })
    }

    /// The claims in the wiki's "Revenue Clearing Time Weight" example
    #[test]
    fn matches_wiki_worked_example() {
        let (channel_epoch, channel_epoch_final) = finalized(90);
        let alice = position(ALICE, 200, &channel_epoch);
        let bob = position(BOB, 90, &channel_epoch);

        let alice_claim = claim_by(alice, channel_epoch, channel_epoch_final).unwrap();
        let bob_claim = claim_by(
            bob,
            alice_claim.channel_epoch_next,
            alice_claim.channel_epoch_final_next,
        )
        .unwrap();

        // floor(90 * 200 / 290) = 62 and floor(90 * 90 / 290) = 27. One reward unit stays in the pot.
        assert_eq!(alice_claim.reward_earned, 62);
        assert_eq!(bob_claim.reward_earned, 27);
        assert!(alice_claim.bidder_position_next.lots.is_empty());

        let channel_epoch_final = &bob_claim.channel_epoch_final_next;
        assert_eq!(channel_epoch_final.unclaimed_weighted_shares, shares(0));
    }

    #[test]
    fn claim_order_does_not_change_payouts() {
        let (channel_epoch, channel_epoch_final) = finalized(90);
        let alice = position(ALICE, 200, &channel_epoch);
        let bob = position(BOB, 90, &channel_epoch);

        let bob_claim = claim_by(bob, channel_epoch, channel_epoch_final).unwrap();
        let alice_claim = claim_by(
            alice,
            bob_claim.channel_epoch_next,
            bob_claim.channel_epoch_final_next,
        )
        .unwrap();

        assert_eq!(bob_claim.reward_earned, 27);
        assert_eq!(alice_claim.reward_earned, 62);
    }

    #[test]
    fn zero_reward_claim_still_retires_shares() {
        // floor(1 * 90 / 290) = 0
        let (channel_epoch, channel_epoch_final) = finalized(1);
        let bob = position(BOB, 90, &channel_epoch);

        let bob_claim = claim_by(bob, channel_epoch, channel_epoch_final).unwrap();

        assert_eq!(bob_claim.reward_earned, 0);
        let channel_epoch_final = &bob_claim.channel_epoch_final_next;
        assert_eq!(channel_epoch_final.unclaimed_weighted_shares, shares(200));
    }

    #[test]
    fn rejects_second_claim() {
        let (channel_epoch, channel_epoch_final) = finalized(90);
        let alice = position(ALICE, 200, &channel_epoch);

        let first = claim_by(alice, channel_epoch, channel_epoch_final).unwrap();
        let second = claim_by(
            first.bidder_position_next,
            first.channel_epoch_next,
            first.channel_epoch_final_next,
        );

        assert_eq!(second.err(), Some(E::NoSharesToClaim));
    }

    #[test]
    fn rejects_final_snapshot_from_another_channel_epoch() {
        let (channel_epoch, mut channel_epoch_final) = finalized(90);
        channel_epoch_final.channel_epoch_id.epoch_counter = 4;
        let alice = position(ALICE, 200, &channel_epoch);

        let result = claim_by(alice, channel_epoch, channel_epoch_final);
        assert_eq!(result.err(), Some(E::ChannelEpochFinalMismatch));
    }
}
