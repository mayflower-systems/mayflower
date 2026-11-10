mod fixture;
mod minting;
mod runtime;
mod tokens;

pub use fixture::Fixture;
pub use minting::MintingSnapshot;
pub use runtime::{Bank, Snapshot, ix_error, meta_mut};

pub use mayf_weighted_emitter_sol_cpi as cpi;
pub use solana_address::{Address, address};
pub use solana_instruction::Instruction;
pub use solana_instruction_error::InstructionError;
pub use solana_keypair::Keypair;
pub use solana_signer::Signer;
pub use solana_transaction_error::TransactionError;

pub const PROGRAM_ID: Address = address!("6tdyEnBkiNYwbF5FmiF4Dq8piqq16CA642JEkMS88RgL");
pub const API: cpi::MayfWeightedEmitterApi = cpi::MayfWeightedEmitterApi::new(PROGRAM_ID);
pub const START: i64 = 1_800_000_000;
pub const RATE: u64 = 100;
pub const INITIAL_SUPPLY: u64 = 1_000_000;
pub const USER_TOKENS: u64 = 10_000;

pub fn pda(seeds: &[&[u8]]) -> Address {
    Address::find_program_address(seeds, &PROGRAM_ID).0
}
