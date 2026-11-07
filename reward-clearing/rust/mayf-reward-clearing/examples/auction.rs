//! The wiki's "Revenue Clearing Time Weight" example.
//!
//! A channel opens a channel epoch. Alice deposits at the start. Bob deposits halfway and withdraws 40 credits.
//! A funder adds 90 reward units to the pot. The channel epoch is finalized,
//! and then Alice and Bob claim.

use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::NonNeg;
use mayf_reward_clearing::models::Identifiable;
use mayf_reward_clearing::models::bidder_position::BidderPosition;
use mayf_reward_clearing::models::channel::Channel;
use mayf_reward_clearing::models::epoch::Epoch;
use mayf_reward_clearing::{
    AddRewardArgs, ClaimArgs, CreateChannelEpochArgs, DepositCreditsArgs, EpochTerms,
    FinalizeChannelEpochArgs, RewardClearingError, Seconds, WithdrawCreditsArgs, add_reward, claim,
    create_channel_epoch, deposit_credits, finalize_channel_epoch, withdraw_credits,
};

#[derive(Clone, Copy, PartialEq)]
struct Id(u8);
impl Identifiable for Id {}

const CHANNEL_GROUP: Id = Id(0);
const START: Seconds = 0;
const DURATION: Seconds = 100;
const HALFWAY: Seconds = 50;

fn epoch() -> Epoch<Id> {
    Epoch::new(CHANNEL_GROUP, 0, START, DURATION)
}

fn channel() -> Channel<Id, U64F64> {
    Channel {
        id: Id(1),
        channel_group_id: CHANNEL_GROUP,
        reward_id: Id(2),
        // Ramp bonus slope 1 and exit penalty exponent 2
        default_terms: EpochTerms::new(U64F64::ONE, 2).expect("the reference terms are valid"),
        last_seen_epoch_counter: None,
    }
}

fn main() -> Result<(), RewardClearingError> {
    let channel_epoch = create_channel_epoch(CreateChannelEpochArgs {
        channel: channel(),
        now: START,
        active_epoch: epoch(),
    })?
    .new_channel_epoch;

    let alice = BidderPosition {
        id: Id(10),
        channel_epoch_id: channel_epoch.id(),
        lots: Vec::new(),
        claimed: false,
    };
    let bob = BidderPosition {
        id: Id(11),
        channel_epoch_id: channel_epoch.id(),
        lots: Vec::new(),
        claimed: false,
    };

    // Alice deposits 100 credits at weight 2.
    let alice_deposit = deposit_credits(DepositCreditsArgs {
        bidder_position: alice,
        channel: channel(),
        channel_epoch,
        epoch: epoch(),
        now: START,
        credits_to_deposit: 100,
    })?;

    // Bob deposits 100 credits at weight 1.5.
    let bob_deposit = deposit_credits(DepositCreditsArgs {
        bidder_position: bob,
        channel: channel(),
        channel_epoch: alice_deposit.channel_epoch_next,
        epoch: epoch(),
        now: HALFWAY,
        credits_to_deposit: 100,
    })?;

    // Bob withdraws 40 credits at a 25% penalty.
    let bob_withdrawal = withdraw_credits(WithdrawCreditsArgs {
        bidder_position: bob_deposit.bidder_position_next,
        channel: channel(),
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
        channel: channel(),
        channel_epoch: bob_withdrawal.channel_epoch_next,
        now: HALFWAY,
        amount: 90,
    })?;

    let finalized = finalize_channel_epoch(FinalizeChannelEpochArgs {
        channel: channel(),
        channel_epoch: funded.channel_epoch_next,
        epoch: epoch(),
        now: START + DURATION,
    })?;

    // The host burns the 160 credits left in the channel epoch.
    let channel_epoch = finalized.channel_epoch_next;
    assert_eq!(channel_epoch.clearing_credits_balance, 160);

    // Alice claims floor(200 * 90 / 290) = 62.
    let alice_claim = claim(ClaimArgs {
        bidder_position: alice_deposit.bidder_position_next,
        channel_epoch,
    })?;

    // Bob claims floor(90 * 90 / 290) = 27.
    let bob_claim = claim(ClaimArgs {
        bidder_position: bob_withdrawal.bidder_position_next,
        channel_epoch: alice_claim.channel_epoch_next,
    })?;

    assert_eq!(alice_claim.reward_earned, 62);
    assert_eq!(bob_claim.reward_earned, 27);

    // Every share has claimed, so the last reward unit can go to the protocol.
    let channel_epoch = bob_claim.channel_epoch_next;
    let shares = |whole| NonNeg::<U64F64>::from_u64(whole).unwrap();
    assert_eq!(channel_epoch.weighted_shares_balance, shares(0));
    assert_eq!(channel_epoch.reward_pot_balance, 1);

    println!("Alice claims 62 and Bob claims 27 of the 90 reward units.");
    println!("The last reward unit stays in the pot for the protocol.");
    Ok(())
}
