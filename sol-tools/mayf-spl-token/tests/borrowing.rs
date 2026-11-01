use anchor_lang::{prelude::*, solana_program::program_error::ProgramError};
use mayf_spl_token::{AuthorityType, burn, close_account, set_authority, transfer};

fn with_borrowed_account(test: impl FnOnce(AccountInfo<'_>)) {
    let key = Pubkey::new_unique();
    let mut lamports = 1;
    let mut data = [0; 8];
    let account = AccountInfo::new(
        &key,
        true,
        true,
        &mut lamports,
        &mut data,
        &mayf_spl_token::ID,
        false,
    );
    let _borrow = account.try_borrow_data().unwrap();
    test(account.clone());
}

#[test]
fn empty_movements_do_not_borrow_accounts_or_invoke_spl() {
    with_borrowed_account(|account| {
        transfer(account.clone(), account.clone(), account.clone(), 0, &[]).unwrap();
        burn(account.clone(), account.clone(), account, 0, &[]).unwrap();
    });
}

#[test]
fn effects_preserve_anchors_account_borrow_errors() {
    with_borrowed_account(|account| {
        let results = [
            transfer(account.clone(), account.clone(), account.clone(), 1, &[]),
            burn(account.clone(), account.clone(), account.clone(), 1, &[]),
            close_account(account.clone(), account.clone(), account.clone(), &[]),
            set_authority(
                account.clone(),
                account,
                AuthorityType::MintTokens,
                None,
                &[],
            ),
        ];
        for result in results {
            assert_eq!(result, Err(ProgramError::AccountBorrowFailed.into()));
        }
    });
}
