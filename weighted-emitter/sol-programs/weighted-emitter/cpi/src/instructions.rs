//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]
use borsh::{BorshDeserialize, BorshSerialize};
use solana_address::{Address, address};
use solana_instruction::{AccountMeta, Instruction};

use crate::{InstructionArgs, MayfWeightedEmitterPda, models::*};

// fixed addresses ---------------------------------------------------

pub const SYSTEM_PROGRAM_ID: Address = address!("11111111111111111111111111111111");
pub const TOKEN_PROGRAM_ID: Address = address!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

// instruction payloads ----------------------------------------------

/// Instruction data, one type per instruction, each carrying that
/// instruction's discriminator.
pub mod args {
    use super::*;

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateEmitter {
        pub args: CreateEmitterArgs,
    }

    impl InstructionArgs for CreateEmitter {
        const DISCRIMINATOR: [u8; 8] = [119, 238, 254, 153, 168, 36, 22, 201];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateGauge;

    impl InstructionArgs for CreateGauge {
        const DISCRIMINATOR: [u8; 8] = [36, 92, 148, 102, 185, 224, 141, 126];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateVotePosition;

    impl InstructionArgs for CreateVotePosition {
        const DISCRIMINATOR: [u8; 8] = [209, 143, 132, 123, 199, 109, 170, 166];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct DepositVote {
        pub amount: u64,
    }

    impl InstructionArgs for DepositVote {
        const DISCRIMINATOR: [u8; 8] = [242, 79, 103, 5, 170, 216, 43, 103];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SetEmitterDecayRate {
        pub args: SetEmitterDecayRateArgs,
    }

    impl InstructionArgs for SetEmitterDecayRate {
        const DISCRIMINATOR: [u8; 8] = [55, 248, 227, 148, 156, 232, 95, 53];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SettleEmission;

    impl InstructionArgs for SettleEmission {
        const DISCRIMINATOR: [u8; 8] = [206, 121, 221, 190, 111, 232, 51, 13];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct WithdrawVote {
        pub amount: u64,
    }

    impl InstructionArgs for WithdrawVote {
        const DISCRIMINATOR: [u8; 8] = [243, 255, 70, 200, 3, 242, 103, 137];
    }
}

// instruction catalog -----------------------------------------------

/// Identity of every instruction in the IDL, for dispatch and coverage.
/// Payloads are encoded by [`args`], not by this enum's ordinal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MayfWeightedEmitterInstruction {
    CreateEmitter,
    CreateGauge,
    CreateVotePosition,
    DepositVote,
    SetEmitterDecayRate,
    SettleEmission,
    WithdrawVote,
}

impl MayfWeightedEmitterInstruction {
    pub const ALL: &'static [Self] = &[
        Self::CreateEmitter,
        Self::CreateGauge,
        Self::CreateVotePosition,
        Self::DepositVote,
        Self::SetEmitterDecayRate,
        Self::SettleEmission,
        Self::WithdrawVote,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::CreateEmitter => "create_emitter",
            Self::CreateGauge => "create_gauge",
            Self::CreateVotePosition => "create_vote_position",
            Self::DepositVote => "deposit_vote",
            Self::SetEmitterDecayRate => "set_emitter_decay_rate",
            Self::SettleEmission => "settle_emission",
            Self::WithdrawVote => "withdraw_vote",
        }
    }

    pub const fn discriminator(self) -> [u8; 8] {
        match self {
            Self::CreateEmitter => args::CreateEmitter::DISCRIMINATOR,
            Self::CreateGauge => args::CreateGauge::DISCRIMINATOR,
            Self::CreateVotePosition => args::CreateVotePosition::DISCRIMINATOR,
            Self::DepositVote => args::DepositVote::DISCRIMINATOR,
            Self::SetEmitterDecayRate => args::SetEmitterDecayRate::DISCRIMINATOR,
            Self::SettleEmission => args::SettleEmission::DISCRIMINATOR,
            Self::WithdrawVote => args::WithdrawVote::DISCRIMINATOR,
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

    /// Accounts for `create_emitter`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateEmitter {
        pub payer: Address,
        pub authority: Address,
        pub emitter: Address,
        pub vote_mint: Address,
        pub emission_mint: Address,
        pub mint_authority: Address,
    }

    impl CreateEmitter {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            payer: Address,
            authority: Address,
            emitter: Address,
            vote_mint: Address,
            emission_mint: Address,
            mint_authority: Address,
        ) -> Self {
            Self {
                payer,
                authority,
                emitter,
                vote_mint,
                emission_mint,
                mint_authority,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new(self.emitter, true),
                AccountMeta::new_readonly(self.vote_mint, false),
                AccountMeta::new_readonly(self.emission_mint, false),
                AccountMeta::new_readonly(self.mint_authority, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ]
        }
    }

    /// Accounts for `create_gauge`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateGauge {
        pub payer: Address,
        pub authority: Address,
        pub emitter: Address,
        pub gauge: Address,
    }

    impl CreateGauge {
        /// Build the list from caller-chosen accounts.
        pub fn new(payer: Address, authority: Address, emitter: Address, gauge: Address) -> Self {
            Self {
                payer,
                authority,
                emitter,
                gauge,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new(self.emitter, false),
                AccountMeta::new(self.gauge, true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ]
        }
    }

    /// Accounts for `create_vote_position`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateVotePosition {
        pub payer: Address,
        pub authority: Address,
        pub emitter: Address,
        pub gauge: Address,
        pub vote_position: Address,
        pub vote_mint: Address,
        pub vote_escrow: Address,
    }

    impl CreateVotePosition {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            payer: Address,
            authority: Address,
            emitter: Address,
            gauge: Address,
            vote_position: Address,
            vote_mint: Address,
            vote_escrow: Address,
        ) -> Self {
            Self {
                payer,
                authority,
                emitter,
                gauge,
                vote_position,
                vote_mint,
                vote_escrow,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new_readonly(self.emitter, false),
                AccountMeta::new_readonly(self.gauge, false),
                AccountMeta::new(self.vote_position, false),
                AccountMeta::new_readonly(self.vote_mint, false),
                AccountMeta::new(self.vote_escrow, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ]
        }
    }

    /// Accounts for `deposit_vote`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct DepositVote {
        pub authority: Address,
        pub emitter: Address,
        pub gauge: Address,
        pub vote_position: Address,
        pub vote_mint: Address,
        pub vote_escrow: Address,
        pub vote_source: Address,
    }

    impl DepositVote {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            authority: Address,
            emitter: Address,
            gauge: Address,
            vote_position: Address,
            vote_mint: Address,
            vote_escrow: Address,
            vote_source: Address,
        ) -> Self {
            Self {
                authority,
                emitter,
                gauge,
                vote_position,
                vote_mint,
                vote_escrow,
                vote_source,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new(self.emitter, false),
                AccountMeta::new(self.gauge, false),
                AccountMeta::new(self.vote_position, false),
                AccountMeta::new_readonly(self.vote_mint, false),
                AccountMeta::new(self.vote_escrow, false),
                AccountMeta::new(self.vote_source, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ]
        }
    }

    /// Accounts for `set_emitter_decay_rate`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SetEmitterDecayRate {
        pub authority: Address,
        pub emitter: Address,
    }

    impl SetEmitterDecayRate {
        /// Build the list from caller-chosen accounts.
        pub fn new(authority: Address, emitter: Address) -> Self {
            Self { authority, emitter }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new(self.emitter, false),
            ]
        }
    }

    /// Accounts for `settle_emission`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SettleEmission {
        pub authority: Address,
        pub emitter: Address,
        pub gauge: Address,
        pub emission_mint: Address,
        pub mint_authority: Address,
        /// The emitter authority chooses the recipient; its token owner is unrestricted.
        pub emission_destination: Address,
    }

    impl SettleEmission {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            authority: Address,
            emitter: Address,
            gauge: Address,
            emission_mint: Address,
            mint_authority: Address,
            emission_destination: Address,
        ) -> Self {
            Self {
                authority,
                emitter,
                gauge,
                emission_mint,
                mint_authority,
                emission_destination,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new(self.emitter, false),
                AccountMeta::new(self.gauge, false),
                AccountMeta::new(self.emission_mint, false),
                AccountMeta::new_readonly(self.mint_authority, false),
                AccountMeta::new(self.emission_destination, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ]
        }
    }

    /// Accounts for `withdraw_vote`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct WithdrawVote {
        pub authority: Address,
        pub emitter: Address,
        pub gauge: Address,
        pub vote_position: Address,
        pub vote_mint: Address,
        pub vote_escrow: Address,
        pub vote_destination: Address,
    }

    impl WithdrawVote {
        /// Build the list from caller-chosen accounts.
        pub fn new(
            authority: Address,
            emitter: Address,
            gauge: Address,
            vote_position: Address,
            vote_mint: Address,
            vote_escrow: Address,
            vote_destination: Address,
        ) -> Self {
            Self {
                authority,
                emitter,
                gauge,
                vote_position,
                vote_mint,
                vote_escrow,
                vote_destination,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new(self.emitter, false),
                AccountMeta::new(self.gauge, false),
                AccountMeta::new(self.vote_position, false),
                AccountMeta::new_readonly(self.vote_mint, false),
                AccountMeta::new(self.vote_escrow, false),
                AccountMeta::new(self.vote_destination, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ]
        }
    }
}

// instruction builders ----------------------------------------------

/// Instruction builders scoped to one program deployment.
pub trait MayfWeightedEmitterInstructions: MayfWeightedEmitterPda {
    /// Build the `create_emitter` instruction.
    fn create_emitter(
        &self,
        accounts: &self::accounts::CreateEmitter,
        args: CreateEmitterArgs,
    ) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::CreateEmitter { args }.to_instruction_data(),
        }
    }

    /// Build the `create_gauge` instruction.
    fn create_gauge(&self, accounts: &self::accounts::CreateGauge) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::CreateGauge.to_instruction_data(),
        }
    }

    /// Build the `create_vote_position` instruction.
    fn create_vote_position(&self, accounts: &self::accounts::CreateVotePosition) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::CreateVotePosition.to_instruction_data(),
        }
    }

    /// Build the `deposit_vote` instruction.
    fn deposit_vote(&self, accounts: &self::accounts::DepositVote, amount: u64) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::DepositVote { amount }.to_instruction_data(),
        }
    }

    /// Build the `set_emitter_decay_rate` instruction.
    fn set_emitter_decay_rate(
        &self,
        accounts: &self::accounts::SetEmitterDecayRate,
        args: SetEmitterDecayRateArgs,
    ) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::SetEmitterDecayRate { args }.to_instruction_data(),
        }
    }

    /// Build the `settle_emission` instruction.
    fn settle_emission(&self, accounts: &self::accounts::SettleEmission) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::SettleEmission.to_instruction_data(),
        }
    }

    /// Build the `withdraw_vote` instruction.
    fn withdraw_vote(&self, accounts: &self::accounts::WithdrawVote, amount: u64) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::WithdrawVote { amount }.to_instruction_data(),
        }
    }
}

impl<T: MayfWeightedEmitterPda + ?Sized> MayfWeightedEmitterInstructions for T {}
