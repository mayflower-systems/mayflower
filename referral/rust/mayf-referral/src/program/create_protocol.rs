use core::num::NonZeroU128;

use mayf_fixed_number::{DirectedRounding::Floor, FixedNumber};

use crate::{Amount, Emission, Error, Gauge, NetworkTerms, Protocol, Result};

pub struct CreateProtocolArgs<N> {
    pub terms: NetworkTerms,
    /// Initial cumulative emitter snapshot. Earlier scheduled emission is skipped.
    pub emission: Emission<N>,
}

pub struct CreateProtocolReceipt<N> {
    pub new_protocol: Protocol<N>,
}

/// Start an empty network. The host supplies cumulative emission from its
/// authenticated, continuously decaying schedule; it is not a token balance.
/// Choose a wide accounting driver: lifetime emission can exceed `u64::MAX`.
pub fn create_protocol<N: FixedNumber>(
    args: CreateProtocolArgs<N>,
) -> Result<CreateProtocolReceipt<N>> {
    let grid = NonZeroU128::new(1 << 64).ok_or(Error::NumberOutOfRange)?;
    if N::checked_from_ratio_round(1, grid, Floor)? == N::ZERO {
        return Err(Error::GridTooCoarse);
    }
    // Whole balances and rational-rate intermediates must fit. Lifetime bounds
    // remain a deployment choice; all subsequent arithmetic is checked.
    N::checked_from_u64(Amount::MAX)?.checked_mul_floor(N::checked_from_u64(1_000_000_000_000)?)?;
    args.emission.validate_after(Emission::default())?;
    Ok(CreateProtocolReceipt {
        new_protocol: Protocol {
            terms: args.terms,
            lane_count: 0,
            gauge: Gauge {
                emission_seen: args.emission,
                ..Gauge::default()
            },
        },
    })
}
