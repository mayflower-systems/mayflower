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
    /// Rent and business balances are isolated from transaction fees.
    fee_payer: Keypair,
    pub payer: Keypair,
}

impl Bank {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new();
        let artifact =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../bin/mayf_weighted_emitter_sol.so");
        svm.add_program_from_file(PROGRAM_ID, &artifact)
            .unwrap_or_else(|error| {
                panic!(
                    "cannot load {}: {error}; run ../build.sh",
                    artifact.display()
                )
            });
        let caller_artifact =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("bin/mayf_weighted_emitter_test_caller.so");
        svm.add_program_from_file(mayf_weighted_emitter_test_caller::ID, &caller_artifact)
            .unwrap_or_else(|error| {
                panic!(
                    "cannot load {}: {error}; run ../build.sh",
                    caller_artifact.display()
                )
            });
        let mut bank = Self {
            svm,
            fee_payer: Keypair::new(),
            payer: Keypair::new(),
        };
        bank.fund(&bank.fee_payer.pubkey());
        bank.fund(&bank.payer.pubkey());
        bank.set_time(START);
        bank
    }

    pub fn fund(&mut self, key: &Address) {
        self.svm.airdrop(key, 10_000_000_000).unwrap();
    }

    pub fn set_time(&mut self, at: i64) {
        let mut clock = self.svm.get_sysvar::<Clock>();
        clock.unix_timestamp = at;
        self.svm.set_sysvar(&clock);
    }

    pub fn account(&self, key: &Address) -> Account {
        self.svm
            .get_account(key)
            .unwrap_or_else(|| panic!("missing {key}"))
    }

    pub fn read<T: cpi::AccountData>(&self, key: &Address) -> T {
        let account = self.account(key);
        assert_eq!(account.owner, PROGRAM_ID);
        T::try_from_account_data(&account.data).unwrap()
    }

    #[allow(clippy::result_large_err)]
    fn execute(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
    ) -> Result<TransactionMetadata, FailedTransactionMetadata> {
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
            .unwrap_or_else(|error| panic!("{error:#?}"))
    }

    /// Exact rejection plus byte-for-byte rollback of every supplied account,
    /// including absent new accounts, mint supply and token balances.
    pub fn reject(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
        expected: TransactionError,
    ) {
        let fee_key = self.fee_payer.pubkey();
        let before = Snapshot::capture(
            self,
            instructions
                .iter()
                .flat_map(|ix| &ix.accounts)
                .map(|meta| meta.pubkey)
                .filter(|key| *key != fee_key),
        );
        let fee_before = self.account(&fee_key).lamports;
        let failure = self
            .execute(instructions, signers)
            .expect_err("transaction must fail");
        assert_eq!(failure.err, expected, "{failure:#?}");
        before.assert_unchanged(self);
        assert_eq!(
            self.account(&fee_key).lamports,
            fee_before - failure.meta.fee
        );
    }
}

pub struct Snapshot(BTreeMap<Address, Option<Account>>);

impl Snapshot {
    pub fn capture(bank: &Bank, keys: impl IntoIterator<Item = Address>) -> Self {
        Self(
            keys.into_iter()
                .map(|key| (key, bank.svm.get_account(&key)))
                .collect(),
        )
    }

    pub fn assert_unchanged(&self, bank: &Bank) {
        for (key, before) in &self.0 {
            assert_eq!(&bank.svm.get_account(key), before, "account {key} changed");
        }
    }
}

pub fn meta_mut(ix: &mut Instruction, key: Address) -> &mut AccountMeta {
    ix.accounts
        .iter_mut()
        .find(|meta| meta.pubkey == key)
        .expect("missing account meta")
}

pub fn ix_error(index: u8, code: u32) -> TransactionError {
    TransactionError::InstructionError(index, InstructionError::Custom(code))
}
