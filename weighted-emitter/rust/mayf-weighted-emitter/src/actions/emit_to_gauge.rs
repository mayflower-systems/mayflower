use mayf_fixed_number::IntegralRounding;

use crate::{Emitter, Gauge, GaugeReceipt, Result, Seconds};

pub struct EmitToGaugeArgs<I, N: IntegralRounding> {
    pub emitter: Emitter<I, N>,
    pub gauge: Gauge<I, N>,
    pub at: Seconds,
}

/// Allocate whole base units and permanently truncate the gauge's fraction.
/// Settlement authorization and cadence are host policy.
pub fn emit_to_gauge<I: Copy + Eq, N: IntegralRounding>(
    args: EmitToGaugeArgs<I, N>,
) -> Result<GaugeReceipt<I, N>> {
    super::settle(args.emitter, args.gauge, args.at)
}
