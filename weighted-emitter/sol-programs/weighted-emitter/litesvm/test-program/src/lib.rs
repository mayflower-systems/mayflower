#![allow(unexpected_cfgs)]

//! Test-only caller that signs for its PDA while invoking the real emitter.
//! Intentionally permissionless; never use this fixture as a production authority.

use anchor_lang::solana_program::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub const ID: Pubkey = Pubkey::new_from_array([42; 32]);
pub const AUTHORITY_SEED: &[u8] = b"emitter_authority";
pub const EMITTER_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("6tdyEnBkiNYwbF5FmiF4Dq8piqq16CA642JEkMS88RgL");

#[cfg(not(feature = "no-entrypoint"))]
anchor_lang::solana_program::entrypoint!(process_instruction);

pub fn authority() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[AUTHORITY_SEED], &ID)
}

/// Preserve the generated target instruction, granting its authority signer
/// privilege only inside the CPI through this fixture's seeds.
pub fn wrap(mut instruction: Instruction) -> Instruction {
    let (authority, _) = authority();
    let mut accounts = vec![
        AccountMeta::new_readonly(authority, false),
        AccountMeta::new_readonly(instruction.program_id, false),
    ];
    for meta in &mut instruction.accounts {
        if meta.pubkey == authority {
            meta.is_signer = false;
        }
    }
    accounts.extend(instruction.accounts);
    Instruction {
        program_id: ID,
        accounts,
        data: instruction.data,
    }
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    let (authority, bump) = authority();
    if program_id != &ID || accounts.len() < 2 {
        return Err(ProgramError::InvalidArgument);
    }
    if accounts[0].key != &authority
        || accounts[1].key != &EMITTER_PROGRAM_ID
        || !accounts[1].executable
    {
        return Err(ProgramError::IncorrectProgramId);
    }
    let instruction = Instruction {
        program_id: *accounts[1].key,
        accounts: accounts[2..]
            .iter()
            .map(|account| AccountMeta {
                pubkey: *account.key,
                is_signer: account.is_signer || account.key == &authority,
                is_writable: account.is_writable,
            })
            .collect(),
        data: data.to_vec(),
    };
    invoke_signed(&instruction, &accounts[1..], &[&[AUTHORITY_SEED, &[bump]]])
}
