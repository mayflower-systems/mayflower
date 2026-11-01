# mayf-characteristic-linear

Persistable neutral parameters for the linear characteristic
`f(x) = slope * x`.

This crate owns parameter validation only. It deliberately does not implement
point evaluation, integration, inversion, or affine transforms. The production
calculator is in `mayf-characteristics-prod`.

Only a positive slope is representable here. A zero or negative slope is
rejected at construction. Constructing a concrete `Characteristic` is the
layer that asserts the complete increasing-function contract.
