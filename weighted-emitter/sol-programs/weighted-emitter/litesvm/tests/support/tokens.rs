use solana_account::Account;
use solana_program_pack::Pack;
use spl_token_interface::{
    instruction, native_mint,
    state::{Account as TokenAccount, AccountState, Mint},
};

use super::*;
use cpi::instructions::TOKEN_PROGRAM_ID;

impl Bank {
    fn seed_token_data<T: Pack>(
        &mut self,
        address: Address,
        value: T,
        extra_lamports: u64,
    ) -> Address {
        let mut data = vec![0; T::LEN];
        T::pack(value, &mut data).unwrap();
        self.svm
            .set_account(
                address,
                Account {
                    lamports: self.svm.minimum_balance_for_rent_exemption(data.len())
                        + extra_lamports,
                    owner: TOKEN_PROGRAM_ID,
                    data,
                    ..Account::default()
                },
            )
            .unwrap();
        address
    }

    pub fn mint(&mut self, authority: Address) -> Address {
        self.seed_token_data(
            Address::new_unique(),
            Mint {
                mint_authority: Some(authority.to_bytes().into()).into(),
                freeze_authority: Some(authority.to_bytes().into()).into(),
                supply: INITIAL_SUPPLY,
                decimals: 6,
                is_initialized: true,
            },
            0,
        )
    }

    pub fn native_mint(&mut self) -> Address {
        self.seed_token_data(
            native_mint::ID,
            Mint {
                decimals: native_mint::DECIMALS,
                is_initialized: true,
                ..Mint::default()
            },
            0,
        )
    }

    pub fn token_account(&mut self, mint: Address, owner: Address, amount: u64) -> Address {
        let native = mint == native_mint::ID;
        let reserve = self
            .svm
            .minimum_balance_for_rent_exemption(TokenAccount::LEN);
        self.seed_token_data(
            Address::new_unique(),
            TokenAccount {
                mint: mint.to_bytes().into(),
                owner: owner.to_bytes().into(),
                amount,
                state: AccountState::Initialized,
                is_native: native.then_some(reserve).into(),
                ..TokenAccount::default()
            },
            if native { amount } else { 0 },
        )
    }

    pub fn tokens(&self, key: &Address) -> TokenAccount {
        let account = self.account(key);
        assert_eq!(account.owner, TOKEN_PROGRAM_ID);
        TokenAccount::unpack(&account.data).unwrap()
    }

    pub fn supply(&self, key: &Address) -> u64 {
        Mint::unpack(&self.account(key).data).unwrap().supply
    }

    pub fn assign_mint_authority(&mut self, mint: Address, current: &Keypair, next: Address) {
        let ix = instruction::set_authority(
            &TOKEN_PROGRAM_ID,
            &mint,
            Some(&next),
            instruction::AuthorityType::MintTokens,
            &current.pubkey(),
            &[],
        )
        .unwrap();
        self.send(&[ix], &[current]);
    }

    pub fn freeze(&mut self, account: Address, mint: Address, authority: &Keypair) {
        let ix = instruction::freeze_account(
            &TOKEN_PROGRAM_ID,
            &account,
            &mint,
            &authority.pubkey(),
            &[],
        )
        .unwrap();
        self.send(&[ix], &[authority]);
    }
}
