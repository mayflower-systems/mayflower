use anchor_lang::prelude::*;

#[constant]
pub const ORGANIZATION_SEED: &[u8] = b"organization";

#[constant]
pub const CHANNEL_GROUP_SEED: &[u8] = b"channel_group";

#[constant]
pub const CLEARING_TOKEN_ESCROW_SEED: &[u8] = b"clearing_token_escrow";

#[constant]
pub const REWARD_CHANNEL_SEED: &[u8] = b"reward_channel";

#[constant]
pub const REWARD_STAGING_SEED: &[u8] = b"reward_staging_token_escrow";

#[constant]
pub const EPOCH_SEED: &[u8] = b"epoch";

#[constant]
pub const CHANNEL_EPOCH_SEED: &[u8] = b"channel_epoch";

#[constant]
pub const CHANNEL_EPOCH_REWARD_POT_ESCROW_SEED: &[u8] = b"channel_epoch_reward_pot_escrow";

#[constant]
pub const CHANNEL_EPOCH_CLEARING_CREDITS_ESCROW_SEED: &[u8] = b"channel_epoch_credits_escrow";

#[constant]
pub const BIDDER_POSITION_SEED: &[u8] = b"bidder_position";

#[constant]
pub const MAX_LOT_COUNT: u32 = 8;
