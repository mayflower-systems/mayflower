use mayf_fixed_number::IntegralRounding;

use crate::{Amount, Emitter, Gauge, GaugeReceipt, Result, Seconds, WeightedEmitterError};

pub struct SetGaugeVotesArgs<I, N: IntegralRounding> {
    pub emitter: Emitter<I, N>,
    pub gauge: Gauge<I, N>,
    /// New aggregate gauge weight, authenticated and backed by the host.
    pub votes: Amount,
    pub at: Seconds,
}

/// Settle this gauge at its old weight before replacing its contribution to the
/// emitter's total. Individual voter records and custody are host concerns.
pub fn set_gauge_votes<I: Copy + Eq, N: IntegralRounding>(
    args: SetGaugeVotesArgs<I, N>,
) -> Result<GaugeReceipt<I, N>> {
    let mut receipt = super::settle(args.emitter, args.gauge, args.at)?;
    receipt.emitter_next.total_gauge_weight = receipt
        .emitter_next
        .total_gauge_weight
        .checked_sub(receipt.gauge_next.votes)
        .and_then(|weight| weight.checked_add(args.votes))
        .ok_or(WeightedEmitterError::ArithmeticOverflow)?;
    receipt.gauge_next.votes = args.votes;
    Ok(receipt)
}
