use solana_account::Account;
use solana_program_pack::Pack;
use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint};

use super::{Address, Bank, cpi::instructions::TOKEN_PROGRAM_ID};

impl Bank {
    fn seed_token_data<T: Pack>(&mut self, value: T) -> Address {
        let address = Address::new_unique();
        let mut data = vec![0; T::LEN];
        T::pack(value, &mut data).unwrap();
        self.svm
            .set_account(
                address,
                Account {
                    lamports: self.svm.minimum_balance_for_rent_exemption(data.len()),
                    owner: TOKEN_PROGRAM_ID,
                    data,
                    ..Account::default()
                },
            )
            .unwrap();
        address
    }

    pub fn mint(&mut self, supply: u64) -> Address {
        self.seed_token_data(Mint {
            supply,
            decimals: 0,
            is_initialized: true,
            ..Mint::default()
        })
    }

    pub fn token_account(&mut self, mint: Address, owner: Address, amount: u64) -> Address {
        self.seed_token_data(TokenAccount {
            mint: mint.to_bytes().into(),
            owner: owner.to_bytes().into(),
            amount,
            state: AccountState::Initialized,
            ..TokenAccount::default()
        })
    }

    pub fn tokens(&self, address: &Address) -> TokenAccount {
        let account = self.account(address);
        assert_eq!(account.owner, TOKEN_PROGRAM_ID);
        TokenAccount::unpack(&account.data).unwrap()
    }

    pub fn supply(&self, address: &Address) -> u64 {
        Mint::unpack(&self.account(address).data).unwrap().supply
    }
}
