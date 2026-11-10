use super::*;

/// Check the mint CPI against persisted accounting, without predicting emission
/// from time, rates, weights, or indexes. Those calculations belong to the library.
pub struct MintingSnapshot {
    emitter: Address,
    gauge: Address,
    mint: Address,
    destination: Address,
    emitter_minted: u64,
    gauge_minted: u64,
    supply: u64,
    destination_balance: u64,
}

impl MintingSnapshot {
    pub fn capture(
        bank: &Bank,
        emitter: Address,
        gauge: Address,
        mint: Address,
        destination: Address,
    ) -> Self {
        Self {
            emitter,
            gauge,
            mint,
            destination,
            emitter_minted: bank.read::<cpi::Emitter>(&emitter).total_minted,
            gauge_minted: bank.read::<cpi::Gauge>(&gauge).total_minted,
            supply: bank.supply(&mint),
            destination_balance: bank.tokens(&destination).amount,
        }
    }

    pub fn assert_minted(self, bank: &Bank) -> u64 {
        let emitter: cpi::Emitter = bank.read(&self.emitter);
        let gauge: cpi::Gauge = bank.read(&self.gauge);
        let minted = gauge.total_minted.checked_sub(self.gauge_minted).unwrap();
        assert!(minted > 0, "fixture must exercise the mint CPI");
        assert_eq!(gauge.total_minted, gauge.total_emitted);
        assert_eq!(emitter.total_minted, self.emitter_minted + minted);
        assert_eq!(bank.supply(&self.mint), self.supply + minted);
        assert_eq!(
            bank.tokens(&self.destination).amount,
            self.destination_balance + minted
        );
        minted
    }
}
