use solana_address::Address;

use super::PROGRAM_ID;

/// The client intentionally leaves state-dependent PDA derivation to callers.
pub fn pda(seeds: &[&[u8]]) -> (Address, u8) {
    Address::find_program_address(seeds, &PROGRAM_ID)
}

#[derive(Clone, Copy, Debug)]
pub struct Addresses {
    pub organization: Address,
    pub group: Address,
    pub group_credits: Address,
    pub epoch: Address,
    pub credit_mint: Address,
    pub reward_mint: Address,
    pub channel: Address,
    pub staging: Address,
    pub channel_epoch: Address,
    pub reward_pot: Address,
    pub credits: Address,
    pub position: Address,
}

impl Addresses {
    pub fn new(
        admin: Address,
        bidder: Address,
        credit_mint: Address,
        reward_mint: Address,
    ) -> Self {
        let organization = pda(&[b"organization", admin.as_ref()]).0;
        let group = pda(&[
            b"channel_group",
            organization.as_ref(),
            &0_u32.to_le_bytes(),
        ])
        .0;
        let channel = pda(&[b"reward_channel", group.as_ref(), reward_mint.as_ref()]).0;
        Self {
            organization,
            group,
            group_credits: pda(&[b"clearing_token_escrow", group.as_ref()]).0,
            epoch: Self::numbered(b"epoch", group, 0),
            credit_mint,
            reward_mint,
            channel,
            staging: pda(&[b"reward_staging_token_escrow", channel.as_ref()]).0,
            channel_epoch: Self::numbered(b"channel_epoch", channel, 0),
            reward_pot: Self::numbered(b"channel_epoch_reward_pot_escrow", channel, 0),
            credits: Self::numbered(b"channel_epoch_credits_escrow", channel, 0),
            position: pda(&[
                b"bidder_position",
                channel.as_ref(),
                &0_u32.to_le_bytes(),
                bidder.as_ref(),
            ])
            .0,
        }
    }

    fn numbered(seed: &[u8], parent: Address, index: u32) -> Address {
        pda(&[seed, parent.as_ref(), &index.to_le_bytes()]).0
    }

    pub fn epoch_at(&self, index: u32) -> Address {
        Self::numbered(b"epoch", self.group, index)
    }

    pub fn channel_epoch_at(&self, index: u32) -> Address {
        Self::numbered(b"channel_epoch", self.channel, index)
    }

    pub fn reward_pot_at(&self, index: u32) -> Address {
        Self::numbered(b"channel_epoch_reward_pot_escrow", self.channel, index)
    }

    pub fn credits_at(&self, index: u32) -> Address {
        Self::numbered(b"channel_epoch_credits_escrow", self.channel, index)
    }
}
