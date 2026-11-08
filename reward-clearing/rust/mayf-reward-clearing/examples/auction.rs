//! The wiki's "Revenue Clearing Time Weight" example.
//!
//! Creating a channel also opens its first channel epoch. Alice deposits at the start.
//! Bob deposits halfway and withdraws 40 credits.
//! A funder adds 90 reward units to the pot. The channel epoch is finalized,
//! and then Alice and Bob claim.

use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::NonNeg;
use mayf_reward_clearing::models::Identifiable;
use mayf_reward_clearing::{
    AddRewardArgs, ClaimArgs, CreateBidderPositionArgs, CreateChannelArgs, CreateChannelGroupArgs,
    DepositCreditsArgs, EpochTerms, FinalizeChannelEpochArgs, MIN_EPOCH_DURATION_SECONDS,
    RewardClearingError, Seconds, WithdrawCreditsArgs, add_reward, claim, create_bidder_position,
    create_channel, create_channel_group, deposit_credits, finalize_channel_epoch,
    withdraw_credits,
};

#[derive(Clone, Copy, PartialEq)]
struct Id(u8);
impl Identifiable for Id {}

const CHANNEL_GROUP: Id = Id(0);
const START: Seconds = 0;
const DURATION: Seconds = MIN_EPOCH_DURATION_SECONDS;
const HALFWAY: Seconds = START + DURATION / 2;

fn main() -> Result<(), RewardClearingError> {
    let created_group = create_channel_group(CreateChannelGroupArgs {
        channel_group_id: CHANNEL_GROUP,
        master_start_time_unix_seconds: START,
        epoch_duration_seconds: DURATION,
    })?;
    let group = created_group.new_channel_group;
    let epoch = created_group.initial_epoch;
    let created = create_channel(CreateChannelArgs {
        channel_id: Id(1),
        reward_id: Id(2),
        // Ramp bonus slope 1 and exit penalty exponent 2
        default_terms: EpochTerms::new(U64F64::ONE, 2)?,
        channel_group: group.clone(),
        now: START,
        active_epoch: epoch.clone(),
    })?;
    let channel = created.new_channel;
    let channel_epoch = created.new_channel_epoch;

    let alice = create_bidder_position(CreateBidderPositionArgs {
        position_id: Id(10),
        channel_group: group.clone(),
        channel: channel.clone(),
        channel_epoch: channel_epoch.clone(),
        epoch: epoch.clone(),
        now: START,
    })?
    .bidder_position;

    // Alice deposits 100 credits at weight 2.
    let alice_deposit = deposit_credits(DepositCreditsArgs {
        channel_group: group.clone(),
        max_lot_count: 8,
        bidder_position: alice,
        channel: channel.clone(),
        channel_epoch,
        epoch: epoch.clone(),
        now: START,
        credits_to_deposit: 100,
    })?;

    // Bob deposits 100 credits at weight 1.5.
    let bob = create_bidder_position(CreateBidderPositionArgs {
        position_id: Id(11),
        channel_group: group.clone(),
        channel: channel.clone(),
        channel_epoch: alice_deposit.channel_epoch_next.clone(),
        epoch: epoch.clone(),
        now: HALFWAY,
    })?
    .bidder_position;
    let bob_deposit = deposit_credits(DepositCreditsArgs {
        channel_group: group.clone(),
        max_lot_count: 8,
        bidder_position: bob,
        channel: channel.clone(),
        channel_epoch: alice_deposit.channel_epoch_next,
        epoch: epoch.clone(),
        now: HALFWAY,
        credits_to_deposit: 100,
    })?;

    // Bob withdraws 40 credits at a 25% penalty.
    let bob_withdrawal = withdraw_credits(WithdrawCreditsArgs {
        channel_group: group.clone(),
        bidder_position: bob_deposit.bidder_position_next,
        channel: channel.clone(),
        channel_epoch: bob_deposit.channel_epoch_next,
        epoch: epoch.clone(),
        now: HALFWAY,
        credits_to_withdraw: 40,
    })?;
    assert_eq!(bob_withdrawal.credits_returned, 30);
    assert_eq!(bob_withdrawal.credits_forfeited, 10);

    // A funder adds 90 reward units, after the host takes its funding fee.
    let funded = add_reward(AddRewardArgs {
        channel_group: group,
        epoch: epoch.clone(),
        channel: channel.clone(),
        channel_epoch: bob_withdrawal.channel_epoch_next,
        now: HALFWAY,
        amount: 90,
    })?;

    let finalized = finalize_channel_epoch(FinalizeChannelEpochArgs {
        channel,
        channel_epoch: funded.channel_epoch_next,
        epoch,
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
