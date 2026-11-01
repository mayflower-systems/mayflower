# mayf-characteristic-linear-driver

Point evaluation and integration for the linear characteristic `f(x) = slope*x`,
with a strictly positive slope. `LinearCharacteristic` implements
`Characteristic`; its disposable `LinearCalculator` implements
`MonotoneCalculator`, including the trait's default area solvers.

The contracts live in `mayf-characteristic` and `mayf-monotone`. Persistable
parameters and slope validation live in `mayf-characteristic-linear`; this crate
re-exports `LinearParams` and `LinearParamsError` and owns the calculation formulas.

## Affine views

For horizontal scale `b > 0`, translation `c`, vertical scale `a >= 0`, and
anchor `(x0, y0)`, the shared affine contract is

```text
u(x) = b*x + c
g(x) = y0 + a*(f(u(x)) - f(u(x0)))
```

For a linear characteristic this simplifies to

```text
k = slope*b*a
g(x) = y0 + k*(x - x0)
integral(lo, width) = width*(g(lo) + g(lo + width))/2
```

Translation cancels algebraically. The calculator still checks that `x >= x0`
and that the transformed anchor and evaluated inputs are nonnegative and
representable. It subtracts the exact view coordinates before multiplying, so
rounding two translated coordinates cannot amplify cancellation errors.
A zero vertical scale gives the constant anchor output.

The anchor output may itself be an enclosure. Its uncertainty propagates into
both point and area results, and evaluating a valid anchor returns that enclosure
unchanged. Calculators retain their own affine state; the characteristic keeps
only its neutral parameters.

## Example

```rust
use mayf_characteristic::{AffineAnchor, AffineTransform, Characteristic};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{Bounds, NonNeg, Pos};
use mayf_monotone::MonotoneCalculator;

let n = |value: i64| NonNeg::new(I64F64::from(value)).unwrap();
let characteristic = LinearCharacteristic::new(Pos::<I64F64>::one());
let view = AffineTransform::new(
    Pos::new(n(2).get()).unwrap(), // horizontal scale
    I64F64::from(-2_i64),        // translation
    n(5),                       // vertical scale
    AffineAnchor::exact(n(1), n(32)),
);
let calculator = characteristic.calculator(view);

// g(x) = 32 + 10*(x - 1).
assert_eq!(calculator.y(n(3)).unwrap(), Bounds::exact(n(52)));
assert_eq!(calculator.integrate(n(1), n(2)).unwrap(), Bounds::exact(n(84)));
let buy = calculator
    .max_hi_for_area_at_or_below(n(1), Some(n(3)), n(100))
    .unwrap().unwrap();
assert_eq!(buy.endpoint, n(3));
assert_eq!(buy.area, n(84));
```

## Precision and errors

Calculations require `N: SignedFixed` to support signed translations. No
transcendental capability, floating-point conversion, allocation, or numeric
representation change is needed by the implementation.

The shared `mayf-fixed-number` library supplies checked arithmetic and outward
rounding. Every successful result encloses the exact mathematical value,
including any anchor uncertainty. Chained products can widen the enclosure;
the endpoints need not be the nearest possible floor and ceiling of the entire
formula. An integral of zero width is exactly zero, including outside the view's
domain. Domain failures return `InputOutOfDomain`; unrepresentable checked
intermediates or results return `ArithmeticFailure`.

The implementation factors the quadratic primitive as
`width*((lo-x0)+(lo+width-x0))/2`, avoiding subtraction of large squares.
It splits the bounded affine slope into a whole part and a fractional remainder:
the whole part multiplies width exactly before the fused division by two, while
the remainder multiplies the bounded geometric span. This keeps one-quantum
intervals useful even at large slopes. Each directed area bound increases with
the upper endpoint and decreases with the lower endpoint, preserving the order
needed by the default solvers. Rounding endpoint prices separately and applying
the trapezoid formula would not preserve that order on every grid.

The distance sum, bounded span when used, and other checked intermediates must
fit the numeric representation. The implementation does not promise to cover
every interval whose final exact area alone would fit.

## Verification

Run `cargo test -p mayf-characteristic-linear-driver --locked`.

Tests compare points and integrals against independent, exact arbitrary-size
integer formulas on binary, decimal, and coarse grids. They check containment
and rounding error bounds, including uncertain anchors and signed translations.
Focused regressions cover cancellation, tiny horizontal scales, one-quantum
widths, and intermediate overflow. Small-grid exhaustive searches check the
default area solvers' greatest safe endpoints.
