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
    use crate::models::channel_epoch::ChannelEpochId;
    use crate::test_support::*;

    const ALICE: Id = Id(10);
    const BOB: Id = Id(11);

    fn finalized(reward: Amount, total_shares: u64) -> ChannelEpoch<Id, Number> {
        let mut channel_epoch = channel_epoch();
        channel_epoch.reward_pot_balance = reward;
        channel_epoch.weighted_shares_balance = shares(total_shares);
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
        let channel_epoch = finalized(90, 290);
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
        // (pot, Alice's shares, Bob's shares, Alice's reward, Bob's reward)
        for (pot, alice_shares, bob_shares, alice_reward, bob_reward) in
            [(90, 200, 90, 62, 27), (2, 3, 3, 1, 1)]
        {
            for alice_first in [true, false] {
                let channel_epoch = finalized(pot, alice_shares + bob_shares);
                let frozen = channel_epoch.final_totals;
                let alice = position(ALICE, alice_shares, &channel_epoch);
                let bob = position(BOB, bob_shares, &channel_epoch);
                let (first, second, first_reward, second_reward) = if alice_first {
                    (alice, bob, alice_reward, bob_reward)
                } else {
                    (bob, alice, bob_reward, alice_reward)
                };

                let first = claim_by(first, channel_epoch).unwrap();
                let second = claim_by(second, first.channel_epoch_next).unwrap();

                assert_eq!(first.reward_earned, first_reward);
                assert_eq!(second.reward_earned, second_reward);
                let remaining = second.channel_epoch_next;
                assert_eq!(remaining.weighted_shares_balance, shares(0));
                assert_eq!(
                    remaining.reward_pot_balance,
                    pot - alice_reward - bob_reward
                );
                assert_eq!(remaining.final_totals, frozen);
            }
        }
    }

    #[test]
    fn sole_claimant_receives_the_entire_pot() {
        // Rounding 1/3 to a fixed-point rate before multiplying by 3 loses a unit.
        for pot in [1, u64::MAX] {
            let channel_epoch = finalized(pot, 3);
            let alice = position(ALICE, 3, &channel_epoch);
            let claimed = claim_by(alice, channel_epoch).unwrap();

            assert_eq!(claimed.reward_earned, pot);
            assert_eq!(claimed.channel_epoch_next.reward_pot_balance, 0);
            assert_eq!(
                claimed.channel_epoch_next.weighted_shares_balance,
                shares(0)
            );
        }
    }

    #[test]
    fn zero_reward_claim_still_retires_shares() {
        // floor(90 * 1 / 290) = 0
        for pot in [0, 1] {
            let channel_epoch = finalized(pot, 290);
            let bob = position(BOB, 90, &channel_epoch);

            let bob_claim = claim_by(bob, channel_epoch).unwrap();

            assert_eq!(bob_claim.reward_earned, 0);
            assert!(bob_claim.bidder_position_next.claimed);
            let channel_epoch = &bob_claim.channel_epoch_next;
            assert_eq!(channel_epoch.weighted_shares_balance, shares(200));
            assert_eq!(channel_epoch.reward_pot_balance, pot);
        }
    }

    #[test]
    fn rejects_second_claim() {
        let channel_epoch = finalized(90, 290);
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
        let channel_epoch = finalized(90, 290);
        let mismatches = [
            ChannelEpochId {
                channel_id: Id(9),
                ..channel_epoch.id()
            },
            ChannelEpochId {
                epoch_counter: 4,
                ..channel_epoch.id()
            },
        ];
        for channel_epoch_id in mismatches {
            let mut alice = position(ALICE, 200, &channel_epoch);
            alice.channel_epoch_id = channel_epoch_id;

            let result = claim_by(alice, channel_epoch.clone());
            assert_eq!(result.err(), Some(E::BidderPositionChannelEpochMismatch));
        }
    }

    #[test]
    fn rejects_shares_exceeding_the_unclaimed_balance() {
        let channel_epoch = finalized(1, 3);
        let alice = position(ALICE, 4, &channel_epoch);

        let result = claim_by(alice, channel_epoch);
        assert_eq!(result.err(), Some(E::WeightedSharesUnderflow));
    }
}
