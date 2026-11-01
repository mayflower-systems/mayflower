# mayf-characteristic

Definition seam between persistable characteristic shapes and disposable
calculation views.

## Owns

- `Characteristic`: constructs a calculator for a supplied affine transform.
- `AffineTransform`: horizontal scale/translation, vertical scale, and anchor.
- `AffineAnchor`: a certified point through which the transformed view passes.
- Transform validation errors.

`Characteristic<N>` and `AffineTransform<N>` require `N: SignedFixed` to support
negative translations. The translation is stored directly as `N` rather than
in a sign wrapper; scales and anchor coordinates retain `Pos<N>`/`NonNeg<N>`
constraints. `AffineAnchor<N>` alone needs only `FixedNumber`.

Implementing `Characteristic` asserts that the neutral function passes through
the origin and is increasing, smooth, and differentiable on its supported
non-negative domain.

## State Rule

Persist the neutral characteristic parameters. Do not persist an
`AffineTransform` or calculator. A schedule derives those just in time for the
calculation it needs.

This crate defines no formulas for `y`, integration, or inversion.
Implementations live in separate crates, and each can satisfy the same seam
with its own algorithms.
