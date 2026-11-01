//! SPL Token effects shared by Mayflower's Anchor programs.
//!
//! Pass `&[]` for an instruction signer, or borrowed seed sets for PDA signers.
//! Account handles share storage; these helpers never copy account data. Token
//! validation and errors belong to SPL Token. Zero transfers and burns are
//! no-ops and perform no CPI or token validation.
//!
//! These helpers target the original SPL Token program, with a single authority
//! account. Token-2022 extensions and SPL multisig authorities are not supported.

#![forbid(unsafe_code)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, CloseAccount, SetAuthority, Transfer};

pub use anchor_spl::token::{ID, spl_token::instruction::AuthorityType};

/// Transfer tokens, skipping an empty movement.
pub fn transfer<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    token::transfer(
        CpiContext::new_with_signer(
            ID,
            Transfer {
                from,
                to,
                authority,
            },
            signer_seeds,
        ),
        amount,
    )
}

/// Burn tokens, skipping an empty movement.
pub fn burn<'info>(
    from: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    token::burn(
        CpiContext::new_with_signer(
            ID,
            Burn {
                from,
                mint,
                authority,
            },
            signer_seeds,
        ),
        amount,
    )
}

/// Close a token account and send its lamports to `destination`.
pub fn close_account<'info>(
    account: AccountInfo<'info>,
    destination: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    token::close_account(CpiContext::new_with_signer(
        ID,
        CloseAccount {
            account,
            destination,
            authority,
        },
        signer_seeds,
    ))
}

/// Change or revoke a mint or token-account authority.
pub fn set_authority<'info>(
    account_or_mint: AccountInfo<'info>,
    current_authority: AccountInfo<'info>,
    authority_type: AuthorityType,
    new_authority: Option<Pubkey>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    token::set_authority(
        CpiContext::new_with_signer(
            ID,
            SetAuthority {
                account_or_mint,
                current_authority,
            },
            signer_seeds,
        ),
        authority_type,
        new_authority,
    )
}
