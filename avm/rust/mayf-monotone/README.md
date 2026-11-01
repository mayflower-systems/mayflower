# mayf-monotone

Semantic calculation contract for a non-negative monotone non-decreasing
function.

## Owns

- `MonotoneCalculator`: bounded point evaluation and integration.
- Safe-side endpoint solves for a requested area.
- `MonotoneCalculatorError`, including explicit rejection of negative output.
- Zero-input identity behavior for widths and area targets.

## Does Not Own

A calculator may contain an affine transform and optimized formulas, but this
crate does not define characteristics, persistent parameters, or schedules.
Numeric-driver failures must be mapped before crossing this seam.

The default area solvers are part of the trait contract and can be overridden
by a more precise calculator.
