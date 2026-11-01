# mayf-fixed-dashu

Host-side `FixedNumber` implementation using Dashu for wide intermediate
arithmetic.

Consumers can access this implementation as
`mayf_fixed_drivers::reference::DashuFixed12` by enabling the drivers crate's
`reference` feature, alongside its binary and scaled implementations.

Public values remain signed `i128` numbers on the shared `1e-12` grid. Dashu is
used internally to reduce intermediate overflow and to calculate conservative
transcendental bounds before quantizing back to that grid.

This is a numeric adapter only. It must match the `FixedNumber` contract and
must not acquire AVM domain policy.
