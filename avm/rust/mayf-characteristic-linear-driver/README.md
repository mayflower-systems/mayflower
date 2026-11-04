# mayf-characteristic-linear-driver

`LinearCharacteristic` implements `mayf_characteristic::Characteristic` for the
neutral line `f(x) = slope * x`. It holds the positive slope defined by
`mayf-characteristic-linear` and returns outward-rounded bounds for both
prices and integrals.

```rust
use mayf_characteristic::Characteristic;
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{Bounds, NonNeg, Pos};

let n = |value: i64| NonNeg::new(I64F64::from(value)).unwrap();
let line = LinearCharacteristic::new(Pos::<I64F64>::from_i32(3).unwrap());
assert_eq!(line.y(n(2)).unwrap(), Bounds::exact(n(6)));
// From x = 2 to x = 6 the average price is 12.
assert_eq!(line.integrate(n(2), n(4)).unwrap(), Bounds::exact(n(48)));
```

The integral over `[lo, hi]` is `slope * width * (lo + hi) / 2`. The driver
multiplies the whole part of the slope by the width exactly before evaluating
the trapezoid. Only the fractional part of the slope multiplies an already
rounded area. This keeps bounds tight even for steep lines and one-quantum
intervals. A zero-width integral is exactly zero. If an endpoint, checked
intermediate, or result exceeds the numeric range, the driver can return
`CharacteristicError::ArithmeticFailure`.
