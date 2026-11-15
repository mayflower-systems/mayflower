# mayf-characteristic

A characteristic is the base curve the AVM uses to price issuance and redemption.
Implement `Characteristic<N>` for the immutable parameters of a smooth,
strictly increasing function with `f(0) = 0` on its supported nonnegative
domain.

- `y(x)` encloses the value at a nonnegative progress coordinate. It returns
  exact zero at the origin.
- `integrate(lo, width)` encloses the area from `lo` to `lo + width`. A zero
  width has exactly zero area.

Both return `Bounds<NonNeg<N>>` on success. Round outward so the exact value
lies between the bounds. Return an error for an unsupported input or a result
that cannot be certified. A driver evaluates the base function using only its
parameters and the requested point or interval.
