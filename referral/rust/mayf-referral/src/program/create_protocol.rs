use core::num::NonZeroU128;

use mayf_fixed_number::DirectedRounding::Floor;
use mayf_fixed_number::FixedNumber;

use crate::emission::Emitter;
use crate::weighting::Gauge;
use crate::{Amount, EmissionRate, Error, NetworkTerms, Offer, Protocol, Result, Seconds};

pub struct CreateProtocolArgs {
    pub terms: NetworkTerms,
    pub emission_rate: EmissionRate,
    /// The price of the first top-level seat
    pub top_level_offer: Offer,
    pub now: Seconds,
}

pub struct CreateProtocolReceipt<N> {
    pub new_protocol: Protocol<N>,
}

/// Start a network with no members and no lanes yet. Its emitter runs from `now`, but nothing
/// is emitted until some lane has gauge votes.
///
/// It rejects a number type `N` that rounds in steps coarser than 2^-64, with
/// [`GridTooCoarse`](crate::Error::GridTooCoarse), or that cannot hold every value below 2^64,
/// with [`NumberOutOfRange`](crate::Error::NumberOutOfRange).
pub fn create_protocol<N>(args: CreateProtocolArgs) -> Result<CreateProtocolReceipt<N>>
where
    N: FixedNumber,
{
    // Every value the kernel computes stays below 2^64, and so does every total it rounds against,
    // such as a lane's balance. So N must hold every value below 2^64, and its grid must be at
    // least as fine as 2^-64, which keeps each rounding step under one base unit.
    let two_pow_64 = NonZeroU128::new(1 << 64).expect("2^64 is not zero");
    if N::checked_from_ratio_round(1, two_pow_64, Floor)? == N::ZERO {
        return Err(Error::GridTooCoarse);
    }
    let largest_fraction = N::ONE.checked_sub(N::QUANTUM)?;
    N::checked_from_u64(Amount::MAX)?.checked_add(largest_fraction)?;

    let new_protocol = Protocol {
        terms: args.terms,
        top_level_offer: args.top_level_offer,
        emitter: Emitter::new(args.emission_rate, args.now),
        gauge: Gauge::default(),
    };

    Ok(CreateProtocolReceipt { new_protocol })
}
