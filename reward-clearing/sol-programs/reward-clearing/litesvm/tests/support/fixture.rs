use solana_address::Address;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_signer::Signer;

use super::{API, Addresses, Bank, DURATION, START, TERMS, cpi};
use cpi::{MayfRewardClearingInstructions, instructions::accounts as ix};

pub const INITIAL_TOKENS: u64 = 10_000;

/// Real program initialization, with only external wallets and SPL balances seeded.
pub struct Fixture {
    pub bank: Bank,
    pub admin: Keypair,
    pub bidder: Keypair,
    pub addresses: Addresses,
    pub credit_source: Address,
    pub reward_source: Address,
    pub reward_destination: Address,
}

impl Fixture {
    /// Start before organization creation so creation failures can inspect absent PDAs.
    pub fn empty() -> Self {
        let mut bank = Bank::new();
        let admin = Keypair::new();
        let bidder = Keypair::new();
        bank.fund(&admin.pubkey());
        bank.fund(&bidder.pubkey());
        bank.set_time(START);
        let credit_mint = bank.mint(INITIAL_TOKENS);
        let reward_mint = bank.mint(INITIAL_TOKENS);
        let addresses = Addresses::new(admin.pubkey(), bidder.pubkey(), credit_mint, reward_mint);
        let credit_source = bank.token_account(credit_mint, bidder.pubkey(), INITIAL_TOKENS);
        let reward_source = bank.token_account(reward_mint, admin.pubkey(), INITIAL_TOKENS);
        let reward_destination = bank.token_account(reward_mint, bidder.pubkey(), 0);
        Self {
            bank,
            admin,
            bidder,
            addresses,
            credit_source,
            reward_source,
            reward_destination,
        }
    }

    pub fn new() -> Self {
        let mut fixture = Self::empty();
        fixture.bank.send(
            &[fixture.organization_ix(), fixture.group_ix()],
            &[&fixture.admin],
        );
        fixture
            .bank
            .send(&[fixture.channel_ix()], &[&fixture.admin]);
        fixture
            .bank
            .send(&[fixture.position_ix()], &[&fixture.bidder]);
        fixture
    }

    pub fn organization_ix(&self) -> Instruction {
        API.create_organization(&ix::CreateOrganization {
            payer: self.bank.payer.pubkey(),
            authority: self.admin.pubkey(),
            organization: self.addresses.organization,
        })
    }

    pub fn group_ix(&self) -> Instruction {
        let a = self.addresses;
        API.create_channel_group(
            &ix::CreateChannelGroup {
                payer: self.bank.payer.pubkey(),
                authority: self.admin.pubkey(),
                organization: a.organization,
                channel_group: a.group,
                initial_epoch: a.epoch,
                clearing_credits_mint: a.credit_mint,
                clearing_credits_escrow: a.group_credits,
            },
            cpi::CreateChannelGroupArgs {
                master_start_time_unix_seconds: START as u64,
                epoch_duration_seconds: DURATION,
            },
        )
    }

    pub fn channel_ix(&self) -> Instruction {
        let a = self.addresses;
        API.channel_create(
            &ix::ChannelCreate {
                payer: self.bank.payer.pubkey(),
                authority: self.admin.pubkey(),
                organization: a.organization,
                channel_group: a.group,
                current_epoch: a.epoch,
                reward_token_mint: a.reward_mint,
                clearing_credits_mint: a.credit_mint,
                channel: a.channel,
                reward_staging_token_account: a.staging,
                initial_channel_epoch: a.channel_epoch,
                reward_pot_escrow_account: a.reward_pot,
                clearing_credits_escrow_account: a.credits,
            },
            cpi::ChannelCreateArgs {
                default_epoch_terms: TERMS,
            },
        )
    }

    pub fn position_ix(&self) -> Instruction {
        let a = self.addresses;
        API.bidder_position_create(&ix::BidderPositionCreate {
            payer: self.bank.payer.pubkey(),
            authority: self.bidder.pubkey(),
            channel_group: a.group,
            channel: a.channel,
            epoch: a.epoch,
            channel_epoch: a.channel_epoch,
            bidder_position: a.position,
        })
    }

    pub fn deposit_ix(&self, amount: u64) -> Instruction {
        let a = self.addresses;
        API.bidder_position_deposit(
            &ix::BidderPositionDeposit {
                payer: self.bank.payer.pubkey(),
                authority: self.bidder.pubkey(),
                channel_group: a.group,
                channel: a.channel,
                epoch: a.epoch,
                channel_epoch: a.channel_epoch,
                bidder_position: a.position,
                clearing_credits_escrow_token_account: a.credits,
                clearing_credits_source: self.credit_source,
            },
            amount,
        )
    }

    pub fn withdraw_ix(&self, amount: u64) -> Instruction {
        let a = self.addresses;
        API.bidder_position_withdraw(
            &ix::BidderPositionWithdraw {
                authority: self.bidder.pubkey(),
                channel_group: a.group,
                channel: a.channel,
                epoch: a.epoch,
                channel_epoch: a.channel_epoch,
                bidder_position: a.position,
                clearing_credits_mint: a.credit_mint,
                clearing_credits_escrow_token_account: a.credits,
                clearing_credits_destination: self.credit_source,
            },
            amount,
        )
    }

    pub fn fund_ix(&self, amount: u64) -> Instruction {
        let a = self.addresses;
        API.channel_epoch_add_to_reward_pot(
            &ix::ChannelEpochAddToRewardPot {
                transfer_authority: self.admin.pubkey(),
                channel_group: a.group,
                channel: a.channel,
                epoch: a.epoch,
                channel_epoch: a.channel_epoch,
                reward_pot_escrow_token_account: a.reward_pot,
                reward_token_src: self.reward_source,
            },
            amount,
        )
    }

    pub fn finalize_ix(&self) -> Instruction {
        let a = self.addresses;
        API.channel_epoch_finalize(&ix::ChannelEpochFinalize {
            channel: a.channel,
            completed_epoch: a.epoch,
            completed_channel_epoch: a.channel_epoch,
            clearing_credits_mint: a.credit_mint,
            clearing_credits_escrow_token_account: a.credits,
        })
    }

    pub fn claim_ix(&self) -> Instruction {
        let a = self.addresses;
        API.bidder_position_claim(&ix::BidderPositionClaim {
            authority: self.bidder.pubkey(),
            channel_epoch: a.channel_epoch,
            bidder_position: a.position,
            reward_pot_escrow_token_account: a.reward_pot,
            reward_destination: self.reward_destination,
        })
    }

    pub fn disable_ix(&self, disabled: bool) -> Instruction {
        let a = self.addresses;
        API.channel_set_disabled(
            &ix::ChannelSetDisabled {
                authority: self.admin.pubkey(),
                organization: a.organization,
                channel_group: a.group,
                channel: a.channel,
            },
            cpi::ChannelSetDisabledArgs { disabled },
        )
    }

    pub fn terms_ix(&self, terms: cpi::EpochTerms) -> Instruction {
        let a = self.addresses;
        API.channel_set_default_epoch_terms(
            &ix::ChannelSetDefaultEpochTerms {
                authority: self.admin.pubkey(),
                organization: a.organization,
                channel_group: a.group,
                channel: a.channel,
            },
            cpi::ChannelSetDefaultEpochTermsArgs {
                default_epoch_terms: terms,
            },
        )
    }

    pub fn epoch_ix(&self, next: u32) -> Instruction {
        let a = self.addresses;
        API.epoch_create(&ix::EpochCreate {
            payer: self.bank.payer.pubkey(),
            channel_group: a.group,
            current_epoch: a.epoch_at(next.checked_sub(1).expect("successor epoch")),
            new_epoch: a.epoch_at(next),
        })
    }

    pub fn channel_epoch_ix(&self, index: u32) -> Instruction {
        let a = self.addresses;
        API.channel_epoch_create(&ix::ChannelEpochCreate {
            payer: self.bank.payer.pubkey(),
            channel_group: a.group,
            channel: a.channel,
            current_epoch: a.epoch_at(index),
            new_channel_epoch: a.channel_epoch_at(index),
            reward_token_mint: a.reward_mint,
            clearing_credits_mint: a.credit_mint,
            reward_pot_escrow_account: a.reward_pot_at(index),
            clearing_credits_escrow_account: a.credits_at(index),
        })
    }
}
