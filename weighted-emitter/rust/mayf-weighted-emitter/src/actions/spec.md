The public actions are exported from the crate root and documented in
[`README.md`](../../README.md). All actions return records for atomic host
persistence; emission effects are whole `u64` token base units.

- Create an emitter with a starting rate, continuous-decay approximation, and clock.
- Change its decay prospectively, after accruing the old interval.
- Create a zero-weight gauge attached to the emitter at its current index.
- Materialize a gauge's indexed allocation, rounding down and discarding dust.
- Set an authenticated aggregate gauge weight after settling its old weight.

No fractional carries, upstream closure protocol, or referral dependency is required.
