//! The numerical contract between shared AVM arithmetic and a characteristic engine.

use mayf_characteristic::{Characteristic, CharacteristicError};
use mayf_fixed_number::{Bounds, FixedNumber, NonNeg};

/// An interval on the neutral characteristic's permanent progress axis.
/// Zero width is a point query: both endpoint values are equal and area is zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanRequest<N: FixedNumber> {
    pub lo: NonNeg<N>,
    pub width: NonNeg<N>,
}

/// Certified bounds for one neutral characteristic span, in request order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanResponse<N: FixedNumber> {
    pub y_at_lo: Bounds<NonNeg<N>>,
    pub y_at_hi: Bounds<NonNeg<N>>,
    pub area: Bounds<NonNeg<N>>,
}

impl<N: FixedNumber> SpanResponse<N> {
    pub(crate) fn zero() -> Self {
        let zero = Bounds::exact(NonNeg::zero());
        Self {
            y_at_lo: zero,
            y_at_hi: zero,
            area: zero,
        }
    }
}

/// Failures evaluating or obtaining a batch of neutral characteristic spans.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacteristicEngineError {
    /// Input and output lengths differ.
    InvalidBatch,
    /// The adapter could not complete the engine invocation.
    InvocationFailed,
    Calculation(CharacteristicError),
}

impl From<CharacteristicError> for CharacteristicEngineError {
    fn from(error: CharacteristicError) -> Self {
        Self::Calculation(error)
    }
}

pub type CharacteristicEngineResult<T> = core::result::Result<T, CharacteristicEngineError>;

/// Evaluate an immutable, increasing characteristic passing through `(0, 0)`.
///
/// Implementations write every response in request order and enclose the true
/// endpoint values and integral. Input and output lengths must match.
/// No segmentation, accounting, or policy crosses this
/// boundary. A transport adapter may encode the whole batch into one invocation.
pub trait CharacteristicEngine<N: FixedNumber> {
    fn spans(
        &self,
        requests: &[SpanRequest<N>],
        responses: &mut [SpanResponse<N>],
    ) -> CharacteristicEngineResult<()>;
}

/// Evaluate a batch locally using a [`Characteristic`]'s point and integral methods.
pub struct LocalCharacteristic<C> {
    characteristic: C,
}

impl<C> LocalCharacteristic<C> {
    pub const fn new(characteristic: C) -> Self {
        Self { characteristic }
    }
}

impl<N: FixedNumber, C: Characteristic<N>> CharacteristicEngine<N> for LocalCharacteristic<C> {
    fn spans(
        &self,
        requests: &[SpanRequest<N>],
        responses: &mut [SpanResponse<N>],
    ) -> CharacteristicEngineResult<()> {
        if requests.len() != responses.len() {
            return Err(CharacteristicEngineError::InvalidBatch);
        }
        for (request, response) in requests.iter().zip(responses) {
            let hi = request
                .lo
                .checked_add(request.width)
                .map_err(|_| CharacteristicError::ArithmeticFailure)?;
            let y_at_lo = if request.lo == NonNeg::zero() {
                Bounds::exact(NonNeg::zero())
            } else {
                self.characteristic.y(request.lo)?
            };
            *response = if request.width == NonNeg::zero() {
                SpanResponse {
                    y_at_lo,
                    y_at_hi: y_at_lo,
                    area: Bounds::exact(NonNeg::zero()),
                }
            } else {
                SpanResponse {
                    y_at_lo,
                    y_at_hi: self.characteristic.y(hi)?,
                    area: self.characteristic.integrate(request.lo, request.width)?,
                }
            };
        }
        Ok(())
    }
}
