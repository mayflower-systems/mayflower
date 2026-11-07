//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]
use borsh::{BorshDeserialize, BorshSerialize};
use solana_address::{Address, address};
use solana_instruction::{AccountMeta, Instruction};

use crate::{InstructionArgs, MayfRewardClearingPda, models::*};

// fixed addresses ---------------------------------------------------

pub const TOKEN_PROGRAM_ID: Address = address!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const SYSTEM_PROGRAM_ID: Address = address!("11111111111111111111111111111111");

// instruction payloads ----------------------------------------------

/// Instruction data, one type per instruction, each carrying that
/// instruction's discriminator.
pub mod args {
    use super::*;

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelCreate {
        pub args: ChannelCreateArgs,
    }

    impl InstructionArgs for ChannelCreate {
        const DISCRIMINATOR: [u8; 8] = [213, 253, 72, 169, 211, 191, 116, 189];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelEpochAddToRewardPot {
        pub amount: u64,
    }

    impl InstructionArgs for ChannelEpochAddToRewardPot {
        const DISCRIMINATOR: [u8; 8] = [128, 16, 205, 229, 125, 214, 9, 11];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelEpochCreate;

    impl InstructionArgs for ChannelEpochCreate {
        const DISCRIMINATOR: [u8; 8] = [154, 192, 198, 231, 244, 192, 167, 20];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelEpochFinalize;

    impl InstructionArgs for ChannelEpochFinalize {
        const DISCRIMINATOR: [u8; 8] = [129, 26, 243, 5, 250, 63, 83, 1];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelSetDefaultEpochTerms {
        pub args: ChannelSetDefaultEpochTermsArgs,
    }

    impl InstructionArgs for ChannelSetDefaultEpochTerms {
        const DISCRIMINATOR: [u8; 8] = [176, 169, 203, 1, 170, 74, 8, 150];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelSetDisabled {
        pub args: ChannelSetDisabledArgs,
    }

    impl InstructionArgs for ChannelSetDisabled {
        const DISCRIMINATOR: [u8; 8] = [84, 110, 177, 66, 32, 78, 65, 177];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateOrganization;

    impl InstructionArgs for CreateOrganization {
        const DISCRIMINATOR: [u8; 8] = [60, 173, 177, 39, 122, 23, 68, 185];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateTenant {
        pub args: CreateTenantArgs,
    }

    impl InstructionArgs for CreateTenant {
        const DISCRIMINATOR: [u8; 8] = [97, 97, 222, 226, 173, 96, 6, 160];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EpochCreate;

    impl InstructionArgs for EpochCreate {
        const DISCRIMINATOR: [u8; 8] = [242, 185, 137, 95, 234, 33, 208, 126];
    }
}

// instruction catalog -----------------------------------------------

/// Identity of every instruction in the IDL, for dispatch and coverage.
/// Payloads are encoded by [`args`], not by this enum's ordinal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MayfRewardClearingInstruction {
    ChannelCreate,
    ChannelEpochAddToRewardPot,
    ChannelEpochCreate,
    ChannelEpochFinalize,
    ChannelSetDefaultEpochTerms,
    ChannelSetDisabled,
    CreateOrganization,
    CreateTenant,
    EpochCreate,
}

impl MayfRewardClearingInstruction {
    pub const ALL: &'static [Self] = &[
        Self::ChannelCreate,
        Self::ChannelEpochAddToRewardPot,
        Self::ChannelEpochCreate,
        Self::ChannelEpochFinalize,
        Self::ChannelSetDefaultEpochTerms,
        Self::ChannelSetDisabled,
        Self::CreateOrganization,
        Self::CreateTenant,
        Self::EpochCreate,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::ChannelCreate => "channel_create",
            Self::ChannelEpochAddToRewardPot => "channel_epoch_add_to_reward_pot",
            Self::ChannelEpochCreate => "channel_epoch_create",
            Self::ChannelEpochFinalize => "channel_epoch_finalize",
            Self::ChannelSetDefaultEpochTerms => "channel_set_default_epoch_terms",
            Self::ChannelSetDisabled => "channel_set_disabled",
            Self::CreateOrganization => "create_organization",
            Self::CreateTenant => "create_tenant",
            Self::EpochCreate => "epoch_create",
        }
    }

    pub const fn discriminator(self) -> [u8; 8] {
        match self {
            Self::ChannelCreate => args::ChannelCreate::DISCRIMINATOR,
            Self::ChannelEpochAddToRewardPot => args::ChannelEpochAddToRewardPot::DISCRIMINATOR,
            Self::ChannelEpochCreate => args::ChannelEpochCreate::DISCRIMINATOR,
            Self::ChannelEpochFinalize => args::ChannelEpochFinalize::DISCRIMINATOR,
            Self::ChannelSetDefaultEpochTerms => args::ChannelSetDefaultEpochTerms::DISCRIMINATOR,
            Self::ChannelSetDisabled => args::ChannelSetDisabled::DISCRIMINATOR,
            Self::CreateOrganization => args::CreateOrganization::DISCRIMINATOR,
            Self::CreateTenant => args::CreateTenant::DISCRIMINATOR,
            Self::EpochCreate => args::EpochCreate::DISCRIMINATOR,
        }
    }

    /// Identify a discriminator without decoding or validating its payload.
    pub fn from_discriminator(discriminator: [u8; 8]) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|instruction| instruction.discriminator() == discriminator)
    }
}

// instruction accounts ----------------------------------------------

/// Account lists, one type per instruction, in the order the program
/// expects. Accounts the IDL pins to a fixed address are supplied
/// automatically and are not fields.
pub mod accounts {
    use super::*;

    /// Accounts for `channel_create`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelCreate {
        pub payer: Address,
        pub authority: Address,
        pub organization: Address,
        pub tenant: Address,
        pub reward_token_mint: Address,
        pub channel: Address,
        pub reward_staging_token_account: Address,
    }

    impl ChannelCreate {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            payer: Address,
            authority: Address,
            organization: Address,
            tenant: Address,
            reward_token_mint: Address,
            channel: Address,
            reward_staging_token_account: Address,
        ) -> Self {
            Self {
                payer,
                authority,
                organization,
                tenant,
                reward_token_mint,
                channel,
                reward_staging_token_account,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new_readonly(self.organization, false),
                AccountMeta::new_readonly(self.tenant, false),
                AccountMeta::new_readonly(self.reward_token_mint, false),
                AccountMeta::new(self.channel, false),
                AccountMeta::new(self.reward_staging_token_account, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ]
        }
    }

    /// Accounts for `channel_epoch_add_to_reward_pot`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelEpochAddToRewardPot {
        pub payer: Address,
        pub transfer_authority: Address,
        pub epoch: Address,
        pub channel_epoch: Address,
        pub reward_pot_escrow_token_account: Address,
        pub reward_token_src: Address,
    }

    impl ChannelEpochAddToRewardPot {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            payer: Address,
            transfer_authority: Address,
            epoch: Address,
            channel_epoch: Address,
            reward_pot_escrow_token_account: Address,
            reward_token_src: Address,
        ) -> Self {
            Self {
                payer,
                transfer_authority,
                epoch,
                channel_epoch,
                reward_pot_escrow_token_account,
                reward_token_src,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.transfer_authority, true),
                AccountMeta::new_readonly(self.epoch, false),
                AccountMeta::new(self.channel_epoch, false),
                AccountMeta::new(self.reward_pot_escrow_token_account, false),
                AccountMeta::new(self.reward_token_src, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ]
        }
    }

    /// Accounts for `channel_epoch_create`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelEpochCreate {
        pub payer: Address,
        pub tenant: Address,
        pub channel: Address,
        /// Validated that this is the current epoch for the tenant
        pub current_epoch: Address,
        pub new_channel_epoch: Address,
        pub reward_token_mint: Address,
        pub clearing_credits_mint: Address,
        pub reward_pot_escrow_account: Address,
        pub clearing_credits_escrow_account: Address,
    }

    impl ChannelEpochCreate {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            payer: Address,
            tenant: Address,
            channel: Address,
            current_epoch: Address,
            new_channel_epoch: Address,
            reward_token_mint: Address,
            clearing_credits_mint: Address,
            reward_pot_escrow_account: Address,
            clearing_credits_escrow_account: Address,
        ) -> Self {
            Self {
                payer,
                tenant,
                channel,
                current_epoch,
                new_channel_epoch,
                reward_token_mint,
                clearing_credits_mint,
                reward_pot_escrow_account,
                clearing_credits_escrow_account,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.tenant, false),
                AccountMeta::new_readonly(self.channel, false),
                AccountMeta::new_readonly(self.current_epoch, false),
                AccountMeta::new(self.new_channel_epoch, false),
                AccountMeta::new_readonly(self.reward_token_mint, false),
                AccountMeta::new_readonly(self.clearing_credits_mint, false),
                AccountMeta::new(self.reward_pot_escrow_account, false),
                AccountMeta::new(self.clearing_credits_escrow_account, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ]
        }
    }

    /// Accounts for `channel_epoch_finalize`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelEpochFinalize {
        pub payer: Address,
        pub tenant: Address,
        pub channel: Address,
        pub completed_epoch: Address,
        pub completed_channel_epoch: Address,
        pub channel_epoch_final_snapshot: Address,
        pub clearing_credits_mint: Address,
        pub clearing_credits_escrow_token_account: Address,
    }

    impl ChannelEpochFinalize {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            payer: Address,
            tenant: Address,
            channel: Address,
            completed_epoch: Address,
            completed_channel_epoch: Address,
            channel_epoch_final_snapshot: Address,
            clearing_credits_mint: Address,
            clearing_credits_escrow_token_account: Address,
        ) -> Self {
            Self {
                payer,
                tenant,
                channel,
                completed_epoch,
                completed_channel_epoch,
                channel_epoch_final_snapshot,
                clearing_credits_mint,
                clearing_credits_escrow_token_account,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.tenant, false),
                AccountMeta::new_readonly(self.channel, false),
                AccountMeta::new_readonly(self.completed_epoch, false),
                AccountMeta::new_readonly(self.completed_channel_epoch, false),
                AccountMeta::new(self.channel_epoch_final_snapshot, false),
                AccountMeta::new_readonly(self.clearing_credits_mint, false),
                AccountMeta::new_readonly(self.clearing_credits_escrow_token_account, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ]
        }
    }

    /// Accounts for `channel_set_default_epoch_terms`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelSetDefaultEpochTerms {
        pub authority: Address,
        pub organization: Address,
        pub tenant: Address,
        pub channel: Address,
    }

    impl ChannelSetDefaultEpochTerms {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            authority: Address,
            organization: Address,
            tenant: Address,
            channel: Address,
        ) -> Self {
            Self {
                authority,
                organization,
                tenant,
                channel,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new_readonly(self.organization, false),
                AccountMeta::new_readonly(self.tenant, false),
                AccountMeta::new(self.channel, false),
            ]
        }
    }

    /// Accounts for `channel_set_disabled`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ChannelSetDisabled {
        pub authority: Address,
        pub organization: Address,
        pub tenant: Address,
        pub channel: Address,
    }

    impl ChannelSetDisabled {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            authority: Address,
            organization: Address,
            tenant: Address,
            channel: Address,
        ) -> Self {
            Self {
                authority,
                organization,
                tenant,
                channel,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new_readonly(self.organization, false),
                AccountMeta::new_readonly(self.tenant, false),
                AccountMeta::new(self.channel, false),
            ]
        }
    }

    /// Accounts for `create_organization`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateOrganization {
        pub payer: Address,
        pub authority: Address,
        pub organization: Address,
    }

    impl CreateOrganization {
        /// Build the list from caller-chosen accounts.
        pub fn new(payer: Address, authority: Address, organization: Address) -> Self {
            Self {
                payer,
                authority,
                organization,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new(self.organization, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ]
        }
    }

    /// Accounts for `create_tenant`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateTenant {
        pub payer: Address,
        pub authority: Address,
        pub organization: Address,
        pub tenant: Address,
        pub initial_epoch: Address,
        pub clearing_credits_mint: Address,
        pub clearing_credits_escrow: Address,
    }

    impl CreateTenant {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            payer: Address,
            authority: Address,
            organization: Address,
            tenant: Address,
            initial_epoch: Address,
            clearing_credits_mint: Address,
            clearing_credits_escrow: Address,
        ) -> Self {
            Self {
                payer,
                authority,
                organization,
                tenant,
                initial_epoch,
                clearing_credits_mint,
                clearing_credits_escrow,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new(self.organization, false),
                AccountMeta::new(self.tenant, false),
                AccountMeta::new(self.initial_epoch, false),
                AccountMeta::new_readonly(self.clearing_credits_mint, false),
                AccountMeta::new(self.clearing_credits_escrow, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ]
        }
    }

    /// Accounts for `epoch_create`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EpochCreate {
        pub payer: Address,
        pub tenant: Address,
        pub current_epoch: Address,
        pub new_epoch: Address,
    }

    impl EpochCreate {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            payer: Address,
            tenant: Address,
            current_epoch: Address,
            new_epoch: Address,
        ) -> Self {
            Self {
                payer,
                tenant,
                current_epoch,
                new_epoch,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.tenant, false),
                AccountMeta::new_readonly(self.current_epoch, false),
                AccountMeta::new(self.new_epoch, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ]
        }
    }
}

// instruction builders ----------------------------------------------

/// Instruction builders scoped to one program deployment.
pub trait MayfRewardClearingInstructions: MayfRewardClearingPda {
    /// Build the `channel_create` instruction.
    fn channel_create(
        &self,
        accounts: &self::accounts::ChannelCreate,
        args: ChannelCreateArgs,
    ) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::ChannelCreate { args }.to_instruction_data(),
        }
    }

    /// Build the `channel_epoch_add_to_reward_pot` instruction.
    fn channel_epoch_add_to_reward_pot(
        &self,
        accounts: &self::accounts::ChannelEpochAddToRewardPot,
        amount: u64,
    ) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::ChannelEpochAddToRewardPot { amount }.to_instruction_data(),
        }
    }

    /// Build the `channel_epoch_create` instruction.
    fn channel_epoch_create(&self, accounts: &self::accounts::ChannelEpochCreate) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::ChannelEpochCreate.to_instruction_data(),
        }
    }

    /// Build the `channel_epoch_finalize` instruction.
    fn channel_epoch_finalize(
        &self,
        accounts: &self::accounts::ChannelEpochFinalize,
    ) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::ChannelEpochFinalize.to_instruction_data(),
        }
    }

    /// Build the `channel_set_default_epoch_terms` instruction.
    fn channel_set_default_epoch_terms(
        &self,
        accounts: &self::accounts::ChannelSetDefaultEpochTerms,
        args: ChannelSetDefaultEpochTermsArgs,
    ) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::ChannelSetDefaultEpochTerms { args }.to_instruction_data(),
        }
    }

    /// Build the `channel_set_disabled` instruction.
    fn channel_set_disabled(
        &self,
        accounts: &self::accounts::ChannelSetDisabled,
        args: ChannelSetDisabledArgs,
    ) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::ChannelSetDisabled { args }.to_instruction_data(),
        }
    }

    /// Build the `create_organization` instruction.
    fn create_organization(&self, accounts: &self::accounts::CreateOrganization) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::CreateOrganization.to_instruction_data(),
        }
    }

    /// Build the `create_tenant` instruction.
    fn create_tenant(
        &self,
        accounts: &self::accounts::CreateTenant,
        args: CreateTenantArgs,
    ) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::CreateTenant { args }.to_instruction_data(),
        }
    }

    /// Build the `epoch_create` instruction.
    fn epoch_create(&self, accounts: &self::accounts::EpochCreate) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::EpochCreate.to_instruction_data(),
        }
    }
}

impl<T: MayfRewardClearingPda + ?Sized> MayfRewardClearingInstructions for T {}
