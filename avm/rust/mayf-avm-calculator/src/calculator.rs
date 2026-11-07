//! Assemble segmented prices and integrals from neutral characteristic spans.

use mayf_fixed_number::{AboveOne, Bounds, FixedNumber, NonNeg, Pos, SignedFixed};

use crate::characteristic::{CharacteristicEngine, SpanRequest, SpanResponse};
use crate::{Error, Result, Segmentation};

/// Shared segmentation arithmetic. The characteristic sees only neutral spans.
pub struct ScheduleCalculator<N: SignedFixed> {
    segmentation: Segmentation<N>,
    ramp_scalar: AboveOne<N>,
}

impl<N: SignedFixed> ScheduleCalculator<N> {
    pub const fn new(segmentation: Segmentation<N>, ramp_scalar: AboveOne<N>) -> Self {
        Self {
            segmentation,
            ramp_scalar,
        }
    }

    pub const fn segmentation(&self) -> Segmentation<N> {
        self.segmentation
    }

    pub fn ramp_start(&self) -> Result<NonNeg<N>> {
        self.segmentation.ramp_start()
    }

    pub fn y(
        &self,
        x: NonNeg<N>,
        engine: &dyn CharacteristicEngine<N>,
    ) -> Result<Bounds<NonNeg<N>>> {
        let end = self.segmentation.ramp_end();
        // Main never reads the floor, ramp start, or ramp scalar. In particular,
        // the shared endpoint follows exactly the same evaluation as main.
        if x >= end {
            let at_x = if x == NonNeg::zero() {
                Bounds::exact(NonNeg::zero())
            } else {
                let [point] = evaluate_spans(
                    engine,
                    [SpanRequest {
                        lo: x,
                        width: NonNeg::zero(),
                    }],
                )?;
                point.y_at_lo
            };
            return nonnegative_enclosure(main_price(at_x, self.segmentation.main_offset())?);
        }
        if x <= self.ramp_start()? {
            return Ok(Bounds::exact(self.segmentation.floor()));
        }
        let width = end.checked_sub(x).map_err(|_| Error::ArithmeticFailure)?;
        let [span] = evaluate_spans(engine, [SpanRequest { lo: x, width }])?;
        nonnegative_enclosure(ramp_price(
            span.y_at_lo,
            span.y_at_hi,
            self.segmentation.main_offset(),
            self.ramp_scalar,
        )?)
    }

    /// Enclose the segmented integral. Zero-width and flat spans need no engine.
    pub fn integrate(
        &self,
        lo: NonNeg<N>,
        width: NonNeg<N>,
        engine: &dyn CharacteristicEngine<N>,
    ) -> Result<Bounds<NonNeg<N>>> {
        self.interval(lo, width, false, engine)
            .map(|(area, _)| area)
    }

    /// Quote the sold interval and contract main in the same characteristic batch.
    /// Upward offset rounding preserves the floor join; the quote reserves the
    /// maximum rounding-induced increase in retained ramp backing.
    pub(crate) fn redeem(
        &self,
        remaining: NonNeg<N>,
        progress: NonNeg<N>,
        engine: &dyn CharacteristicEngine<N>,
    ) -> Result<(Bounds<NonNeg<N>>, Segmentation<N>)> {
        let width = progress
            .checked_sub(remaining)
            .map_err(|_| Error::ArithmeticFailure)?;
        self.interval(remaining, width, true, engine)
    }

    fn interval(
        &self,
        lo: NonNeg<N>,
        width: NonNeg<N>,
        contract: bool,
        engine: &dyn CharacteristicEngine<N>,
    ) -> Result<(Bounds<NonNeg<N>>, Segmentation<N>)> {
        let state = self.segmentation;
        if width == NonNeg::zero() {
            return Ok((Bounds::exact(NonNeg::zero()), state));
        }
        let hi = lo
            .checked_add(width)
            .map_err(|_| Error::ArithmeticFailure)?;
        let start = state.ramp_start()?;
        let end = state.ramp_end();
        let flat_width = hi
            .min(start)
            .max(lo)
            .checked_sub(lo)
            .map_err(|_| Error::ArithmeticFailure)?;
        let ramp_lo = lo.max(start);
        let ramp_hi = hi.min(end);
        let ramp_request = span_between(ramp_lo, ramp_hi)?;
        let main_request = span_between(lo.max(end), hi)?;
        let contracting = contract && lo < end;
        // A partial ramp needs f(E). A sale below the ramp instead needs f(lo)
        // to re-anchor the collapsed main. These cases are mutually exclusive.
        let point_x = if ramp_request.is_some() && ramp_hi < end {
            Some(end)
        } else if contracting && (lo < start || ramp_request.is_none()) && lo != NonNeg::zero() {
            Some(lo)
        } else {
            None
        };
        let [ramp, main, point] = evaluate_optional_spans(
            engine,
            [ramp_request, main_request, point_x.map(point_request)],
        )?;
        let mut area = constant_area(state.floor(), flat_width)?.into_raw();
        if let (Some(request), Some(response)) = (ramp_request, ramp) {
            let at_end = if ramp_hi == end {
                response.y_at_hi
            } else {
                point.ok_or(Error::InvalidCharacteristicBatch)?.y_at_lo
            };
            area = area
                .checked_add(ramp_area(
                    response,
                    request.width,
                    at_end,
                    state.main_offset(),
                    self.ramp_scalar,
                )?)
                .map_err(|_| Error::ArithmeticFailure)?;
        }
        if let (Some(request), Some(response)) = (main_request, main) {
            area = area
                .checked_add(main_area(response, request.width, state.main_offset())?)
                .map_err(|_| Error::ArithmeticFailure)?;
        }
        if !contracting {
            return Ok((nonnegative_enclosure(area)?, state));
        }
        let (next, retained_increase) = if lo > start {
            self.contract_ramp(lo, start, ramp.ok_or(Error::InvalidCharacteristicBatch)?)?
        } else {
            let at_lo = if lo == NonNeg::zero() {
                Bounds::exact(NonNeg::zero())
            } else if lo == start && ramp.is_some() {
                ramp.ok_or(Error::InvalidCharacteristicBatch)?.y_at_lo
            } else {
                point.ok_or(Error::InvalidCharacteristicBatch)?.y_at_lo
            };
            (self.collapse(lo, at_lo)?, Bounds::exact(N::ZERO))
        };
        let reserve = area
            .checked_sub(retained_increase)
            .map_err(|_| Error::ArithmeticFailure)?;
        Ok((
            reserve
                .try_nonnegative()
                .map_err(|_| Error::ContractionPrecisionLoss)?,
            next,
        ))
    }

    fn contract_ramp(
        &self,
        progress: NonNeg<N>,
        start: NonNeg<N>,
        sold_ramp: SpanResponse<N>,
    ) -> Result<(Segmentation<N>, Bounds<N>)> {
        let state = self.segmentation;
        let gain = self
            .ramp_scalar
            .get()
            .checked_sub(N::ONE)
            .map_err(|_| Error::ArithmeticFailure)?;
        let change = sold_ramp
            .y_at_lo
            .into_raw()
            .checked_sub(sold_ramp.y_at_hi.into_raw())
            .map_err(|_| Error::ArithmeticFailure)?;
        let offset = scale(change, gain)?
            .checked_add_exact(state.main_offset())
            .map_err(|_| Error::ArithmeticFailure)?;
        // Monotonicity proves this contraction cannot increase main's offset.
        let chosen = offset.upper().min(state.main_offset());
        if chosen < offset.lower() {
            return Err(Error::InvalidCharacteristicBatch);
        }
        let retained_width = progress
            .checked_sub(start)
            .map_err(|_| Error::ArithmeticFailure)?;
        let rounding = chosen
            .checked_sub(offset.lower())
            .map_err(|_| Error::ArithmeticFailure)?;
        let increase = Bounds::product(rounding, retained_width.get())
            .map_err(|_| Error::ArithmeticFailure)?;
        let increase =
            Bounds::new(N::ZERO, increase.upper()).map_err(|_| Error::ArithmeticFailure)?;
        Ok((
            Segmentation::new(state.floor(), progress, retained_width, chosen)?,
            increase,
        ))
    }

    fn collapse(
        &self,
        progress: NonNeg<N>,
        at_progress: Bounds<NonNeg<N>>,
    ) -> Result<Segmentation<N>> {
        let floor = self.segmentation.floor();
        let offset = Bounds::exact(floor.get())
            .checked_sub(at_progress.into_raw())
            .map_err(|_| Error::ArithmeticFailure)?;
        Segmentation::new(floor, progress, NonNeg::zero(), offset.upper())
    }
}

pub(crate) fn point_request<N: FixedNumber>(x: NonNeg<N>) -> SpanRequest<N> {
    SpanRequest {
        lo: x,
        width: NonNeg::zero(),
    }
}

pub(crate) fn span_between<N: FixedNumber>(
    lo: NonNeg<N>,
    hi: NonNeg<N>,
) -> Result<Option<SpanRequest<N>>> {
    if hi <= lo {
        return Ok(None);
    }
    Ok(Some(SpanRequest {
        lo,
        width: hi.checked_sub(lo).map_err(|_| Error::ArithmeticFailure)?,
    }))
}

/// Each slot represents a distinct geometric role; absent and duplicate spans
/// are omitted from the actual synchronous engine invocation.
pub(crate) fn evaluate_optional_spans<N: FixedNumber, const COUNT: usize>(
    engine: &dyn CharacteristicEngine<N>,
    requests: [Option<SpanRequest<N>>; COUNT],
) -> Result<[Option<SpanResponse<N>>; COUNT]> {
    let mut batch = [point_request(NonNeg::zero()); COUNT];
    let mut responses = [SpanResponse::zero(); COUNT];
    let mut indices = [None; COUNT];
    let mut count = 0;
    for (slot, request) in requests.into_iter().enumerate() {
        if let Some(request) = request {
            request
                .lo
                .checked_add(request.width)
                .map_err(|_| Error::ArithmeticFailure)?;
            let index = batch[..count]
                .iter()
                .position(|existing| *existing == request)
                .unwrap_or_else(|| {
                    let index = count;
                    batch[index] = request;
                    count += 1;
                    index
                });
            indices[slot] = Some(index);
        }
    }
    if count > 0 {
        engine.spans(&batch[..count], &mut responses[..count])?;
    }
    Ok(indices.map(|index| index.map(|index| responses[index])))
}

pub(crate) fn evaluate_spans<N: FixedNumber, const COUNT: usize>(
    engine: &dyn CharacteristicEngine<N>,
    requests: [SpanRequest<N>; COUNT],
) -> Result<[SpanResponse<N>; COUNT]> {
    for request in &requests {
        request
            .lo
            .checked_add(request.width)
            .map_err(|_| Error::ArithmeticFailure)?;
    }
    let mut responses = [SpanResponse::zero(); COUNT];
    if COUNT != 0 {
        engine.spans(&requests, &mut responses)?;
    }
    Ok(responses)
}

pub(crate) fn main_price<N: SignedFixed>(at_x: Bounds<NonNeg<N>>, offset: N) -> Result<Bounds<N>> {
    at_x.into_raw()
        .checked_add_exact(offset)
        .map_err(|_| Error::ArithmeticFailure)
}

pub(crate) fn ramp_price<N: SignedFixed>(
    at_x: Bounds<NonNeg<N>>,
    at_end: Bounds<NonNeg<N>>,
    offset: N,
    scalar: AboveOne<N>,
) -> Result<Bounds<N>> {
    // k*f(x) + b - (k-1)*f(E), avoiding a repeated uncertain f(E).
    let gain = scalar
        .get()
        .checked_sub(N::ONE)
        .map_err(|_| Error::ArithmeticFailure)?;
    scale(at_x.into_raw(), scalar.get())?
        .checked_add_exact(offset)
        .map_err(|_| Error::ArithmeticFailure)?
        .checked_sub(scale(at_end.into_raw(), gain)?)
        .map_err(|_| Error::ArithmeticFailure)
}

pub(crate) fn main_area<N: SignedFixed>(
    span: SpanResponse<N>,
    width: NonNeg<N>,
    offset: N,
) -> Result<Bounds<N>> {
    span.area
        .into_raw()
        .checked_add(Bounds::product(offset, width.get()).map_err(|_| Error::ArithmeticFailure)?)
        .map_err(|_| Error::ArithmeticFailure)
}

pub(crate) fn ramp_area<N: SignedFixed>(
    span: SpanResponse<N>,
    width: NonNeg<N>,
    at_end: Bounds<NonNeg<N>>,
    offset: N,
    scalar: AboveOne<N>,
) -> Result<Bounds<N>> {
    let gain = scalar
        .get()
        .checked_sub(N::ONE)
        .map_err(|_| Error::ArithmeticFailure)?;
    let ramp_offset = Bounds::exact(offset)
        .checked_sub(scale(at_end.into_raw(), gain)?)
        .map_err(|_| Error::ArithmeticFailure)?;
    scale(span.area.into_raw(), scalar.get())?
        .checked_add(scale(ramp_offset, width.get())?)
        .map_err(|_| Error::ArithmeticFailure)
}

pub(crate) fn scale<N: SignedFixed>(bounds: Bounds<N>, factor: N) -> Result<Bounds<N>> {
    bounds
        .checked_mul_div(factor, Pos::one())
        .map_err(|_| Error::ArithmeticFailure)
}

pub(crate) fn constant_area<N: FixedNumber>(
    floor: NonNeg<N>,
    width: NonNeg<N>,
) -> Result<Bounds<NonNeg<N>>> {
    (floor.bounded() * width)
        .bounds()
        .map_err(|_| Error::ArithmeticFailure)
}

pub(crate) fn add_bounds<N: FixedNumber>(
    left: Bounds<NonNeg<N>>,
    right: Bounds<NonNeg<N>>,
) -> Result<Bounds<NonNeg<N>>> {
    (left.bounded() + right.bounded())
        .bounds()
        .map_err(|_| Error::ArithmeticFailure)
}

/// Valid schedules have nonnegative prices. Intersect only this universal bound;
/// main enclosures must never depend on the floor or ramp geometry.
pub(crate) fn nonnegative_enclosure<N: SignedFixed>(
    bounds: Bounds<N>,
) -> Result<Bounds<NonNeg<N>>> {
    Bounds::new(bounds.lower().max(N::ZERO), bounds.upper())
        .map_err(|_| Error::NegativeOutput)?
        .try_nonnegative()
        .map_err(|_| Error::NegativeOutput)
}
