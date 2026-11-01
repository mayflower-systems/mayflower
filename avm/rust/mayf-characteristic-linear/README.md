# mayf-characteristic-linear

Persistable neutral parameters for the linear characteristic
`f(x) = slope * x`.

This crate owns parameter validation only. It deliberately does not implement
point evaluation, integration, inversion, or affine transforms. The production
calculator is in `mayf-characteristics-prod`.

A non-negative slope is representable here. Constructing a concrete
`Characteristic` is the layer that asserts the complete increasing-function
contract.
