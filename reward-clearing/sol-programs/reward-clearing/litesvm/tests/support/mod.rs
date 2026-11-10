mod addresses;
mod fixture;
mod runtime;
mod tokens;

pub use addresses::Addresses;
pub use fixture::{Fixture, INITIAL_TOKENS};
pub use runtime::{Bank, Snapshot, ix_error, meta_mut};

pub use cpi::AccountData;
pub use mayf_reward_clearing_sol_cpi as cpi;
pub use solana_address::{Address, address};
pub use solana_instruction::Instruction;
pub use solana_instruction_error::InstructionError;
pub use solana_keypair::Keypair;
pub use solana_signer::Signer;
pub use solana_transaction_error::TransactionError;

pub const PROGRAM_ID: Address = address!("RCgecPmsA1YfbwBZ9FkPmGUkKqnEGwvMrpnzUhUthxd");
pub const API: cpi::MayfRewardClearingApi = cpi::MayfRewardClearingApi::new(PROGRAM_ID);
pub const START: i64 = 1_800_000_000;
pub const DURATION: u32 = 3_600;
// Simple exact shares and a halfway refund let tests assert custody without
// reimplementing the domain's curves or using it as its own expected-value oracle.
pub const TERMS: cpi::EpochTerms = cpi::EpochTerms {
    ramp_bonus_slope_mbps: 0,
    exit_penalty_exponent: 1,
};
