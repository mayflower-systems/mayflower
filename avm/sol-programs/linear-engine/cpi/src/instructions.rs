//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]
use borsh::{BorshDeserialize, BorshSerialize};
use solana_address::{Address, address};
use solana_instruction::{AccountMeta, Instruction};

use crate::{InstructionArgs, MayfAvmEngineLinearPda, models::*};

// fixed addresses ---------------------------------------------------

pub const SYSTEM_PROGRAM_ID: Address = address!("11111111111111111111111111111111");

// instruction payloads ----------------------------------------------

/// Instruction data, one type per instruction, each carrying that
/// instruction's discriminator.
pub mod args {
    use super::*;

    #[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq)]
    pub struct CalculateSpans {
        pub args: CalculateSpansArgs,
    }

    impl InstructionArgs for CalculateSpans {
        const DISCRIMINATOR: [u8; 8] = [153, 97, 174, 93, 213, 190, 207, 24];
    }

    #[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateLinearEngine {
        pub args: CreateLinearEngineArgs,
    }

    impl InstructionArgs for CreateLinearEngine {
        const DISCRIMINATOR: [u8; 8] = [203, 156, 46, 120, 92, 192, 163, 220];
    }
}

// instruction catalog -----------------------------------------------

/// Identity of every instruction in the IDL, for dispatch and coverage.
/// Payloads are encoded by [`args`], not by this enum's ordinal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MayfAvmEngineLinearInstruction {
    CalculateSpans,
    CreateLinearEngine,
}

impl MayfAvmEngineLinearInstruction {
    pub const ALL: &'static [Self] = &[Self::CalculateSpans, Self::CreateLinearEngine];

    pub const fn name(self) -> &'static str {
        match self {
            Self::CalculateSpans => "calculate_spans",
            Self::CreateLinearEngine => "create_linear_engine",
        }
    }

    pub const fn discriminator(self) -> [u8; 8] {
        match self {
            Self::CalculateSpans => args::CalculateSpans::DISCRIMINATOR,
            Self::CreateLinearEngine => args::CreateLinearEngine::DISCRIMINATOR,
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

    /// Accounts for `calculate_spans`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CalculateSpans {
        pub linear_characteristic_param: Address,
    }

    impl CalculateSpans {
        /// Build the list from caller-chosen accounts.
        pub fn new(linear_characteristic_param: Address) -> Self {
            Self {
                linear_characteristic_param,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![AccountMeta::new_readonly(
                self.linear_characteristic_param,
                false,
            )]
        }
    }

    /// Accounts for `create_linear_engine`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreateLinearEngine {
        pub payer: Address,
        pub linear_characteristic_param: Address,
    }

    impl CreateLinearEngine {
        /// Build the list from caller-chosen accounts.
        pub fn new(payer: Address, linear_characteristic_param: Address) -> Self {
            Self {
                payer,
                linear_characteristic_param,
            }
        }

        /// Account metas in the order the program expects.
        pub fn to_account_metas(&self) -> Vec<AccountMeta> {
            vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new(self.linear_characteristic_param, true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ]
        }
    }
}

// instruction builders ----------------------------------------------

/// Instruction builders scoped to one program deployment.
pub trait MayfAvmEngineLinearInstructions: MayfAvmEngineLinearPda {
    /// Evaluate neutral characteristic spans and return their bounds in order.
    /// Build the `calculate_spans` instruction.
    fn calculate_spans(
        &self,
        accounts: &self::accounts::CalculateSpans,
        args: CalculateSpansArgs,
    ) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::CalculateSpans { args }.to_instruction_data(),
        }
    }

    /// Build the `create_linear_engine` instruction.
    fn create_linear_engine(
        &self,
        accounts: &self::accounts::CreateLinearEngine,
        args: CreateLinearEngineArgs,
    ) -> Instruction {
        Instruction {
            program_id: self.program(),
            accounts: accounts.to_account_metas(),
            data: self::args::CreateLinearEngine { args }.to_instruction_data(),
        }
    }
}

impl<T: MayfAvmEngineLinearPda + ?Sized> MayfAvmEngineLinearInstructions for T {}
