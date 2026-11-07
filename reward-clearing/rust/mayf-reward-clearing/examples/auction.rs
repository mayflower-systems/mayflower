//! The wiki's "Revenue Clearing Time Weight" example.
//!
//! Alice deposits at the start. Bob deposits halfway and withdraws 40 credits.
//! A funder adds 90 reward units to the pot. The channel epoch is finalized,
//! and then Alice and Bob claim.

use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::NonNeg;
use mayf_reward_clearing::models::Identifiable;
use mayf_reward_clearing::models::bidder_position::BidderPosition;
use mayf_reward_clearing::models::channel_epoch::ChannelEpoch;
use mayf_reward_clearing::models::epoch::Epoch;
use mayf_reward_clearing::{
    AddRewardArgs, ClaimArgs, DepositCreditsArgs, EpochTerms, FinalizeChannelEpochArgs,
    RewardClearingError, Seconds, WithdrawCreditsArgs, add_reward, claim, deposit_credits,
    finalize_channel_epoch, withdraw_credits,
};

#[derive(Clone, Copy, PartialEq)]
struct Id(u8);
impl Identifiable for Id {}

const START: Seconds = 0;
const DURATION: Seconds = 100;
const HALFWAY: Seconds = 50;

fn epoch() -> Epoch<Id> {
    Epoch::new(Id(0), 0, START, DURATION)
}

fn main() -> Result<(), RewardClearingError> {
    // Ramp bonus slope 1 and exit penalty exponent 2
    let terms = EpochTerms::<U64F64>::new(U64F64::ONE, 2)?;
    let channel_epoch = ChannelEpoch::new(Id(1), 0, terms);

    let alice = BidderPosition {
        id: Id(10),
        channel_epoch_id: channel_epoch.id(),
        lots: Vec::new(),
    };
    let bob = BidderPosition {
        id: Id(11),
        channel_epoch_id: channel_epoch.id(),
        lots: Vec::new(),
    };

    // Alice deposits 100 credits at weight 2.
    let alice_deposit = deposit_credits(DepositCreditsArgs {
        bidder_position: alice,
        channel_epoch,
        epoch: epoch(),
        now: START,
        credits_to_deposit: 100,
    })?;

    // Bob deposits 100 credits at weight 1.5.
    let bob_deposit = deposit_credits(DepositCreditsArgs {
        bidder_position: bob,
        channel_epoch: alice_deposit.channel_epoch_next,
        epoch: epoch(),
        now: HALFWAY,
        credits_to_deposit: 100,
    })?;

    // Bob withdraws 40 credits at a 25% penalty.
    let bob_withdrawal = withdraw_credits(WithdrawCreditsArgs {
        bidder_position: bob_deposit.bidder_position_next,
        channel_epoch: bob_deposit.channel_epoch_next,
        epoch: epoch(),
        now: HALFWAY,
        credits_to_withdraw: 40,
    })?;
    assert_eq!(bob_withdrawal.credits_returned, 30);
    assert_eq!(bob_withdrawal.credits_forfeited, 10);

    // A funder adds 90 reward units, after the host takes its funding fee.
    let funded = add_reward(AddRewardArgs {
        epoch: epoch(),
        channel_epoch: bob_withdrawal.channel_epoch_next,
        now: HALFWAY,
        amount: 90,
    })?;

    let finalized = finalize_channel_epoch(FinalizeChannelEpochArgs {
        channel_epoch: funded.channel_epoch_next,
        epoch: epoch(),
        now: START + DURATION,
    })?;

    let shares = |whole| NonNeg::<U64F64>::from_u64(whole).unwrap();
    assert_eq!(
        finalized.channel_epoch_final.unclaimed_weighted_shares,
        shares(290)
    );

    // The host burns the 160 credits left in the channel epoch.
    assert_eq!(finalized.channel_epoch_next.clearing_credits_balance, 160);

    // Alice claims floor(90 * 200 / 290) = 62.
    let alice_claim = claim(ClaimArgs {
        bidder_position: alice_deposit.bidder_position_next,
        channel_epoch: finalized.channel_epoch_next,
        channel_epoch_final: finalized.channel_epoch_final,
    })?;

    // Bob claims floor(90 * 90 / 290) = 27.
    let bob_claim = claim(ClaimArgs {
        bidder_position: bob_withdrawal.bidder_position_next,
        channel_epoch: alice_claim.channel_epoch_next,
        channel_epoch_final: alice_claim.channel_epoch_final_next,
    })?;

    assert_eq!(alice_claim.reward_earned, 62);
    assert_eq!(bob_claim.reward_earned, 27);
    assert_eq!(
        bob_claim.channel_epoch_final_next.unclaimed_weighted_shares,
        shares(0)
    );

    println!("Alice claims 62 and Bob claims 27 of the 90 reward units.");
    println!("The last reward unit stays in the pot for the protocol.");
    Ok(())
}
