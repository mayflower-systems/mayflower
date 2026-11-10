use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::{Emitter, Result, Seconds};

pub struct SetEmitterDecayRateArgs<I, N: FixedNumber> {
    pub emitter: Emitter<I, N>,
    pub annual_decay_rate: NonNeg<N>,
    pub at: Seconds,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetEmitterDecayRateReceipt<I, N: FixedNumber> {
    pub emitter_next: Emitter<I, N>,
}

/// Finish the old schedule through `at`, then apply the new decay prospectively.
/// Changing decay never resets or increases the current emission rate.
pub fn set_emitter_decay_rate<I, N: FixedNumber>(
    args: SetEmitterDecayRateArgs<I, N>,
) -> Result<SetEmitterDecayRateReceipt<I, N>> {
    let mut emitter_next = super::advance(args.emitter, args.at)?;
    emitter_next.schedule.annual_decay_rate = args.annual_decay_rate;
    Ok(SetEmitterDecayRateReceipt { emitter_next })
}
