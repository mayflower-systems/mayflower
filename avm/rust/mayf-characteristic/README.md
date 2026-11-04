# mayf-characteristic

A characteristic describes how redemption price grows with issuance progress.
Implement `Characteristic<N>` for the immutable parameters of a smooth,
increasing function with `f(0) = 0`. The trait has two methods:

- `y(x)` encloses the value at a nonnegative progress coordinate.
- `integrate(lo, width)` encloses the area from `lo` to `lo + width`. A zero
  width has exactly zero area.

Both return `Bounds<NonNeg<N>>` on success. Round outward so the exact value
lies between the bounds; return an error when it cannot be certified.
The trait works with any `FixedNumber`; each driver chooses how to evaluate its
own function. For example, `mayf-characteristic-linear-driver` implements both
methods for `f(x) = slope * x`.

The AVM calculator uses these neutral values and integrals to place the floor,
scale the ramp, and quote trades. A local driver can be passed to its
`LocalCharacteristic` adapter; a remote engine can answer the same span batches
in one invocation. Neither needs market state to evaluate the characteristic.
