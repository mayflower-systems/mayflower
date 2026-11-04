//! Assemble segmented prices and integrals from neutral characteristic spans.

use mayf_fixed_number::{AboveOne, Bounds, FixedNumber, NonNeg, Pos, SignedFixed};
use mayf_monotone::require_nonnegative;

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
        let start = self.ramp_start()?;
        let end = self.segmentation.ramp_end();
        let floor = self.segmentation.floor();
        if x <= start {
            return Ok(Bounds::exact(floor));
        }

        let price = if x <= end {
            let width = x.checked_sub(start).map_err(|_| Error::ArithmeticFailure)?;
            let [ramp] = evaluate_spans(engine, [SpanRequest { lo: start, width }])?;
            self.ramp_price(ramp.y_at_lo, ramp.y_at_hi)?
        } else if self.segmentation.ramp_width() == NonNeg::zero() {
            let point = SpanRequest {
                lo: x,
                width: NonNeg::zero(),
            };
            let (at_end, at_x) = if end == NonNeg::zero() {
                let [point] = evaluate_spans(engine, [point])?;
                (Bounds::exact(NonNeg::zero()), point.y_at_lo)
            } else {
                let [anchor, point] = evaluate_spans(
                    engine,
                    [
                        SpanRequest {
                            lo: end,
                            width: NonNeg::zero(),
                        },
                        point,
                    ],
                )?;
                (anchor.y_at_lo, point.y_at_lo)
            };
            let growth = at_x
                .into_raw()
                .checked_sub(at_end.into_raw())
                .map_err(|_| Error::ArithmeticFailure)?;
            growth
                .checked_add_exact(floor.get())
                .map_err(|_| Error::ArithmeticFailure)?
        } else {
            let [ramp, point] = evaluate_spans(
                engine,
                [
                    SpanRequest {
                        lo: start,
                        width: self.segmentation.ramp_width(),
                    },
                    SpanRequest {
                        lo: x,
                        width: NonNeg::zero(),
                    },
                ],
            )?;
            let anchor_price = self.ramp_price(ramp.y_at_lo, ramp.y_at_hi)?;
            let growth = point
                .y_at_lo
                .into_raw()
                .checked_sub(ramp.y_at_hi.into_raw())
                .map_err(|_| Error::ArithmeticFailure)?;
            anchor_price
                .checked_add(growth)
                .map_err(|_| Error::ArithmeticFailure)?
        };
        require_nonnegative(floor_enclosure(price, floor)?).map_err(Into::into)
    }

    /// Enclose `[lo, lo + width]`, using only the spans needed by its segments.
    /// A partial-ramp/main interval needs two integrals and the ramp-start point.
    /// Zero width or a wholly flat interval requires no characteristic call.
    pub fn integrate(
        &self,
        lo: NonNeg<N>,
        width: NonNeg<N>,
        engine: &dyn CharacteristicEngine<N>,
    ) -> Result<Bounds<NonNeg<N>>> {
        if width == NonNeg::zero() {
            return Ok(Bounds::exact(NonNeg::zero()));
        }
        let hi = lo
            .checked_add(width)
            .map_err(|_| Error::ArithmeticFailure)?;
        let floor = self.segmentation.floor();
        let minimum = constant_area(floor, width)?.lower();
        let start = self.ramp_start()?;
        let end = self.segmentation.ramp_end();
        if hi <= start {
            return constant_area(floor, width);
        }

        let cursor = lo.max(start);
        let flat_width = cursor
            .checked_sub(lo)
            .map_err(|_| Error::ArithmeticFailure)?;
        let flat_area = constant_area(floor, flat_width)?;
        let curve_area = if cursor >= end {
            let width = hi
                .checked_sub(cursor)
                .map_err(|_| Error::ArithmeticFailure)?;
            self.main_area(cursor, width, engine)?
        } else if hi <= end {
            let width = hi
                .checked_sub(cursor)
                .map_err(|_| Error::ArithmeticFailure)?;
            let request = SpanRequest { lo: cursor, width };
            let (ramp, anchor) = if cursor == start {
                let [ramp] = evaluate_spans(engine, [request])?;
                (ramp, ramp.y_at_lo)
            } else if start == NonNeg::zero() {
                let [ramp] = evaluate_spans(engine, [request])?;
                (ramp, Bounds::exact(NonNeg::zero()))
            } else {
                let [point, ramp] = evaluate_spans(
                    engine,
                    [
                        SpanRequest {
                            lo: start,
                            width: NonNeg::zero(),
                        },
                        request,
                    ],
                )?;
                (ramp, point.y_at_lo)
            };
            anchored_area(
                ramp,
                width,
                anchor,
                Bounds::exact(floor.get()),
                self.ramp_scalar.get(),
            )?
        } else {
            let ramp_width = end
                .checked_sub(cursor)
                .map_err(|_| Error::ArithmeticFailure)?;
            let main_width = hi.checked_sub(end).map_err(|_| Error::ArithmeticFailure)?;
            let ramp_request = SpanRequest {
                lo: cursor,
                width: ramp_width,
            };
            let main_request = SpanRequest {
                lo: end,
                width: main_width,
            };
            let (ramp, main, anchor) = if cursor == start {
                let [ramp, main] = evaluate_spans(engine, [ramp_request, main_request])?;
                (ramp, main, ramp.y_at_lo)
            } else if start == NonNeg::zero() {
                let [ramp, main] = evaluate_spans(engine, [ramp_request, main_request])?;
                (ramp, main, Bounds::exact(NonNeg::zero()))
            } else {
                let [point, ramp, main] = evaluate_spans(
                    engine,
                    [
                        SpanRequest {
                            lo: start,
                            width: NonNeg::zero(),
                        },
                        ramp_request,
                        main_request,
                    ],
                )?;
                (ramp, main, point.y_at_lo)
            };
            let ramp_area = anchored_area(
                ramp,
                ramp_width,
                anchor,
                Bounds::exact(floor.get()),
                self.ramp_scalar.get(),
            )?;
            // The redeemed ramp span already supplies f(E); no full-ramp query
            // is needed to anchor the main segment.
            let end_price = self.ramp_price(anchor, ramp.y_at_hi)?;
            let main_area = anchored_area(main, main_width, ramp.y_at_hi, end_price, N::ONE)?;
            ramp_area
                .checked_add(main_area)
                .map_err(|_| Error::ArithmeticFailure)?
        };
        let area = flat_area
            .into_raw()
            .checked_add(curve_area)
            .map_err(|_| Error::ArithmeticFailure)?;
        require_nonnegative(floor_enclosure(area, minimum)?).map_err(Into::into)
    }

    fn ramp_price(
        &self,
        at_start: Bounds<NonNeg<N>>,
        at_end: Bounds<NonNeg<N>>,
    ) -> Result<Bounds<N>> {
        let growth = at_end
            .into_raw()
            .checked_sub(at_start.into_raw())
            .map_err(|_| Error::ArithmeticFailure)?;
        let price = scale(growth, self.ramp_scalar.get())?
            .checked_add_exact(self.segmentation.floor().get())
            .map_err(|_| Error::ArithmeticFailure)?;
        floor_enclosure(price, self.segmentation.floor())
    }

    fn main_area(
        &self,
        lo: NonNeg<N>,
        width: NonNeg<N>,
        engine: &dyn CharacteristicEngine<N>,
    ) -> Result<Bounds<N>> {
        let start = self.ramp_start()?;
        let end = self.segmentation.ramp_end();
        let floor = Bounds::exact(self.segmentation.floor().get());
        let request = SpanRequest { lo, width };
        if self.segmentation.ramp_width() == NonNeg::zero() {
            let (main, anchor) = if end == NonNeg::zero() {
                let [main] = evaluate_spans(engine, [request])?;
                (main, Bounds::exact(NonNeg::zero()))
            } else if lo == end {
                let [main] = evaluate_spans(engine, [request])?;
                (main, main.y_at_lo)
            } else {
                let [point, main] = evaluate_spans(
                    engine,
                    [
                        SpanRequest {
                            lo: end,
                            width: NonNeg::zero(),
                        },
                        request,
                    ],
                )?;
                (main, point.y_at_lo)
            };
            return anchored_area(main, width, anchor, floor, N::ONE);
        }

        let (main, at_start, at_end) = if lo == end {
            // The main integral already returns f(E), so only f(A) is missing.
            if start == NonNeg::zero() {
                let [main] = evaluate_spans(engine, [request])?;
                (main, Bounds::exact(NonNeg::zero()), main.y_at_lo)
            } else {
                let [point, main] = evaluate_spans(
                    engine,
                    [
                        SpanRequest {
                            lo: start,
                            width: NonNeg::zero(),
                        },
                        request,
                    ],
                )?;
                (main, point.y_at_lo, main.y_at_lo)
            }
        } else {
            let [ramp, main] = evaluate_spans(
                engine,
                [
                    SpanRequest {
                        lo: start,
                        width: self.segmentation.ramp_width(),
                    },
                    request,
                ],
            )?;
            (main, ramp.y_at_lo, ramp.y_at_hi)
        };
        let end_price = self.ramp_price(at_start, at_end)?;
        anchored_area(main, width, at_end, end_price, N::ONE)
    }
}

/// Evaluate an operation's complete request array in one invocation.
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

/// Integrate `base_price + gain * (f(x) - f(anchor))` over the supplied span.
fn anchored_area<N: SignedFixed>(
    span: SpanResponse<N>,
    width: NonNeg<N>,
    anchor: Bounds<NonNeg<N>>,
    base_price: Bounds<N>,
    gain: N,
) -> Result<Bounds<N>> {
    let rectangle = scale(anchor.into_raw(), width.get())?;
    let growth = span
        .area
        .into_raw()
        .checked_sub(rectangle)
        .map_err(|_| Error::ArithmeticFailure)?;
    let scaled_growth = scale(growth, gain)?;
    let base_area = scale(base_price, width.get())?;
    scaled_growth
        .checked_add(base_area)
        .map_err(|_| Error::ArithmeticFailure)
}

/// Geometry proves every schedule price is at least its floor, and therefore
/// every integral is at least its floor rectangle. Intersect with that proved
/// lower bound when independent endpoint enclosures lose their correlation.
/// An upper edge below the bound contradicts the characteristic contract.
pub(crate) fn floor_enclosure<N: SignedFixed>(
    bounds: Bounds<N>,
    minimum: NonNeg<N>,
) -> Result<Bounds<N>> {
    Bounds::new(bounds.lower().max(minimum.get()), bounds.upper())
        .map_err(|_| Error::NegativeOutput)
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
