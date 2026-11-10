use std::{collections::BTreeMap, path::Path};

use litesvm::{
    LiteSVM,
    types::{FailedTransactionMetadata, TransactionMetadata},
};
use solana_account::Account;
use solana_clock::Clock;
use solana_instruction::AccountMeta;
use solana_transaction::Transaction;

use super::*;

pub struct Bank {
    pub svm: LiteSVM,
    /// Pays only transaction fees, so rent refunds and failed rent transfers can
    /// be asserted exactly on the business accounts.
    pub fee_payer: Keypair,
    pub payer: Keypair,
}

impl Bank {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new();
        let artifact =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../bin/mayf_reward_clearing_sol.so");
        svm.add_program_from_file(PROGRAM_ID, &artifact)
            .unwrap_or_else(|error| {
                panic!(
                    "cannot load {}: {error}; run ../build.sh",
                    artifact.display()
                )
            });
        let mut bank = Self {
            svm,
            fee_payer: Keypair::new(),
            payer: Keypair::new(),
        };
        bank.fund(&bank.fee_payer.pubkey());
        bank.fund(&bank.payer.pubkey());
        bank
    }

    pub fn fund(&mut self, address: &Address) {
        self.svm.airdrop(address, 10_000_000_000).unwrap();
    }

    pub fn set_time(&mut self, timestamp: i64) {
        let mut clock = self.svm.get_sysvar::<Clock>();
        clock.unix_timestamp = timestamp;
        self.svm.set_sysvar(&clock);
    }

    pub fn account(&self, address: &Address) -> Account {
        self.svm
            .get_account(address)
            .unwrap_or_else(|| panic!("missing account {address}"))
    }

    pub fn read<T: AccountData>(&self, address: &Address) -> T {
        let account = self.account(address);
        assert_eq!(account.owner, PROGRAM_ID, "owner of {address}");
        T::try_from_account_data(&account.data).unwrap()
    }

    /// Deliberately corrupt stored fields through the same public codec clients
    /// consume. Keep allocation and lamports intact, including retained lot space.
    pub fn overwrite<T: AccountData>(&mut self, address: &Address, value: &T) {
        let mut account = self.account(address);
        let data = value.to_account_data();
        assert!(data.len() <= account.data.len());
        account.data.fill(0);
        account.data[..data.len()].copy_from_slice(&data);
        self.svm.set_account(*address, account).unwrap();
    }

    pub fn assert_rent_exempt(&self, address: &Address) {
        let account = self.account(address);
        assert!(
            account.lamports
                >= self
                    .svm
                    .minimum_balance_for_rent_exemption(account.data.len()),
            "{address} must cover its allocated space"
        );
    }

    // The public test interface keeps failures rich, including all runtime logs.
    #[allow(clippy::result_large_err)]
    fn execute(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
    ) -> Result<TransactionMetadata, FailedTransactionMetadata> {
        // Preserve signature verification and replay protection; use a fresh
        // blockhash so an intentional retry reaches the program again.
        self.svm.expire_blockhash();
        let mut required = vec![&self.fee_payer];
        if instructions
            .iter()
            .flat_map(|ix| &ix.accounts)
            .any(|meta| meta.is_signer && meta.pubkey == self.payer.pubkey())
        {
            required.push(&self.payer);
        }
        for signer in signers {
            if !required
                .iter()
                .any(|known| known.pubkey() == signer.pubkey())
            {
                required.push(signer);
            }
        }
        let tx = Transaction::new_signed_with_payer(
            instructions,
            Some(&self.fee_payer.pubkey()),
            &required,
            self.svm.latest_blockhash(),
        );
        self.svm.send_transaction(tx)
    }

    pub fn send(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
    ) -> TransactionMetadata {
        self.execute(instructions, signers)
            .unwrap_or_else(|failure| panic!("{failure:#?}"))
    }

    /// Every rejection checks the exact failure/index and *all* supplied accounts,
    /// including missing PDAs, mint supply, token balances, allocation and rent.
    pub fn reject(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
        expected: TransactionError,
    ) -> FailedTransactionMetadata {
        let fee_address = self.fee_payer.pubkey();
        let before = Snapshot::capture(
            self,
            instructions
                .iter()
                .flat_map(|ix| &ix.accounts)
                .map(|meta| meta.pubkey)
                .filter(|key| *key != fee_address),
        );
        let fee_balance = self.account(&fee_address).lamports;
        let failure = self
            .execute(instructions, signers)
            .expect_err("transaction must fail");
        assert_eq!(failure.err, expected, "{failure:#?}");
        before.assert_unchanged(self);
        assert_eq!(
            self.account(&fee_address).lamports,
            fee_balance - failure.meta.fee,
            "only the transaction fee may escape rollback"
        );
        failure
    }
}

pub struct Snapshot(BTreeMap<Address, Option<Account>>);

impl Snapshot {
    pub fn capture(bank: &Bank, addresses: impl IntoIterator<Item = Address>) -> Self {
        Self(
            addresses
                .into_iter()
                .map(|address| (address, bank.svm.get_account(&address)))
                .collect(),
        )
    }

    pub fn assert_unchanged(&self, bank: &Bank) {
        for (address, before) in &self.0 {
            assert_eq!(
                &bank.svm.get_account(address),
                before,
                "account {address} changed"
            );
        }
    }
}

pub fn meta_mut(instruction: &mut Instruction, address: Address) -> &mut AccountMeta {
    instruction
        .accounts
        .iter_mut()
        .find(|meta| meta.pubkey == address)
        .expect("instruction must contain the account under attack")
}

pub fn ix_error(index: u8, code: u32) -> TransactionError {
    TransactionError::InstructionError(index, InstructionError::Custom(code))
}
