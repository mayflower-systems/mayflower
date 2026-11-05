use crate::{Epoch, EpochTerms, EpochWindow, Result};
use mayf_fixed_number::FixedNumber;

/// Immutable schedule window and terms for a new auction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartEpochArgs<N: FixedNumber> {
    pub window: EpochWindow,
    pub terms: EpochTerms<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartEpochReceipt<N: FixedNumber> {
    pub epoch: Epoch<N>,
}

pub fn start_epoch<N: FixedNumber>(args: StartEpochArgs<N>) -> Result<StartEpochReceipt<N>> {
    Ok(StartEpochReceipt {
        epoch: Epoch::try_new(args.window, args.terms)?,
    })
}
