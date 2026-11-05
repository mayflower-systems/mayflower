use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ChannelEpochFinal {
    /// Link to channel epoch that is completed
    pub channel_epoch: Pubkey,

    /// Mutable raw Q64.64 shares that have not yet claimed. This is the closure
    /// semaphore: claims decrement it by their exact position shares, and only
    /// zero permits the finalized account and pot to close.
    pub unclaimed_weighted_shares: u128,
}
