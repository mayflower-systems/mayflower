use super::*;
use cpi::{MayfWeightedEmitterInstructions, instructions::accounts as ix};

pub struct Voter {
    pub authority: Keypair,
    pub gauge: Address,
    pub position: Address,
    pub escrow: Address,
    pub source: Address,
}

impl Voter {
    fn new(bank: &mut Bank, gauge: Address, mint: Address) -> Self {
        let authority = Keypair::new();
        bank.fund(&authority.pubkey());
        let position = pda(&[
            b"vote_position",
            gauge.as_ref(),
            authority.pubkey().as_ref(),
        ]);
        let escrow = pda(&[b"vote_escrow", position.as_ref()]);
        let source = bank.token_account(mint, authority.pubkey(), USER_TOKENS);
        Self {
            authority,
            gauge,
            position,
            escrow,
            source,
        }
    }
}

/// Program-owned accounts are initialized by real instructions. Only external
/// mints and funded wallets are seeded, using SPL's public account codecs.
pub struct Fixture {
    pub bank: Bank,
    pub admin: Keypair,
    pub emitter: Keypair,
    pub gauge: Keypair,
    pub mint_authority: Address,
    pub vote_mint: Address,
    pub emission_mint: Address,
    pub voter: Voter,
    pub destination: Address,
}

impl Fixture {
    pub fn empty(same_mint: bool) -> Self {
        let mut bank = Bank::new();
        let admin = Keypair::new();
        bank.fund(&admin.pubkey());
        let emitter = Keypair::new();
        let gauge = Keypair::new();
        let mint_authority = pda(&[b"mint_authority", emitter.pubkey().as_ref()]);
        let emission_mint = bank.mint(admin.pubkey());
        let vote_mint = if same_mint {
            emission_mint
        } else {
            bank.mint(admin.pubkey())
        };
        let voter = Voter::new(&mut bank, gauge.pubkey(), vote_mint);
        // Same-mint deployments bootstrap supply before handing mint authority
        // to the emitter PDA. The handover uses the actual SPL instruction.
        bank.assign_mint_authority(emission_mint, &admin, mint_authority);
        // The recipient deliberately belongs to somebody other than the admin.
        let destination = bank.token_account(emission_mint, voter.authority.pubkey(), 0);
        Self {
            bank,
            admin,
            emitter,
            gauge,
            mint_authority,
            vote_mint,
            emission_mint,
            voter,
            destination,
        }
    }

    pub fn new(same_mint: bool) -> Self {
        let mut f = Self::empty(same_mint);
        f.initialize();
        f
    }

    pub fn initialize(&mut self) {
        self.bank.send(
            &[self.emitter_ix(), self.gauge_ix(&self.gauge)],
            &[&self.admin, &self.emitter, &self.gauge],
        );
        self.bank
            .send(&[self.position_ix(&self.voter)], &[&self.voter.authority]);
    }

    pub fn add_voter(&mut self) -> Voter {
        self.add_voter_for(self.gauge.pubkey())
    }

    pub fn add_voter_for(&mut self, gauge: Address) -> Voter {
        let voter = Voter::new(&mut self.bank, gauge, self.vote_mint);
        self.bank
            .send(&[self.position_ix(&voter)], &[&voter.authority]);
        voter
    }

    pub fn emitter_ix(&self) -> Instruction {
        self.emitter_with_rate_ix(u128::from(RATE) << 64)
    }

    pub fn emitter_with_rate_ix(&self, emission_rate_q64f64: u128) -> Instruction {
        API.create_emitter(
            &ix::CreateEmitter {
                payer: self.bank.payer.pubkey(),
                authority: self.admin.pubkey(),
                emitter: self.emitter.pubkey(),
                vote_mint: self.vote_mint,
                emission_mint: self.emission_mint,
                mint_authority: self.mint_authority,
            },
            cpi::CreateEmitterArgs {
                emission_rate_q64f64,
                annual_decay_rate_q64f64: 0,
            },
        )
    }

    pub fn gauge_ix(&self, gauge: &Keypair) -> Instruction {
        API.create_gauge(&ix::CreateGauge {
            payer: self.bank.payer.pubkey(),
            authority: self.admin.pubkey(),
            emitter: self.emitter.pubkey(),
            gauge: gauge.pubkey(),
        })
    }

    pub fn position_ix(&self, voter: &Voter) -> Instruction {
        API.create_vote_position(&ix::CreateVotePosition {
            payer: self.bank.payer.pubkey(),
            authority: voter.authority.pubkey(),
            emitter: self.emitter.pubkey(),
            gauge: voter.gauge,
            vote_position: voter.position,
            vote_mint: self.vote_mint,
            vote_escrow: voter.escrow,
        })
    }

    pub fn deposit_ix(&self, voter: &Voter, amount: u64) -> Instruction {
        API.deposit_vote(
            &ix::DepositVote {
                authority: voter.authority.pubkey(),
                emitter: self.emitter.pubkey(),
                gauge: voter.gauge,
                vote_position: voter.position,
                vote_mint: self.vote_mint,
                vote_escrow: voter.escrow,
                vote_source: voter.source,
            },
            amount,
        )
    }

    pub fn withdraw_ix(&self, voter: &Voter, amount: u64) -> Instruction {
        API.withdraw_vote(
            &ix::WithdrawVote {
                authority: voter.authority.pubkey(),
                emitter: self.emitter.pubkey(),
                gauge: voter.gauge,
                vote_position: voter.position,
                vote_mint: self.vote_mint,
                vote_escrow: voter.escrow,
                vote_destination: voter.source,
            },
            amount,
        )
    }

    pub fn settle_ix(&self) -> Instruction {
        API.settle_emission(&ix::SettleEmission {
            authority: self.admin.pubkey(),
            emitter: self.emitter.pubkey(),
            gauge: self.gauge.pubkey(),
            emission_mint: self.emission_mint,
            mint_authority: self.mint_authority,
            emission_destination: self.destination,
        })
    }

    pub fn decay_ix(&self, annual_decay_rate_q64f64: u128) -> Instruction {
        API.set_emitter_decay_rate(
            &ix::SetEmitterDecayRate {
                authority: self.admin.pubkey(),
                emitter: self.emitter.pubkey(),
            },
            cpi::SetEmitterDecayRateArgs {
                annual_decay_rate_q64f64,
            },
        )
    }

    pub fn minting_snapshot(&self, gauge: Address, destination: Address) -> MintingSnapshot {
        MintingSnapshot::capture(
            &self.bank,
            self.emitter.pubkey(),
            gauge,
            self.emission_mint,
            destination,
        )
    }

    pub fn state(&self) -> (cpi::Emitter, cpi::Gauge, cpi::VotePosition) {
        (
            self.bank.read(&self.emitter.pubkey()),
            self.bank.read(&self.gauge.pubkey()),
            self.bank.read(&self.voter.position),
        )
    }
}
