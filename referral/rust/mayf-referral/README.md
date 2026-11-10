# mayf-referral

A pure state machine for fixed upline commissions, credit-escrow family marketing,
and rewards within each asset lane. Public actions validate their inputs and
return next state plus custody effects. The host applies each receipt atomically.

The host configures maximum depth `H` and commission `q`, subject to `H*q < 1`.
Both zero depth and zero commission are valid. The current lane mask has storage
capacity for 16 assets; tree height is a separate configuration. Asset changes
touch one ancestor path; family changes touch one parent's source lanes. No
action loads a subtree. Root seats consume external auction awards and return a
credit burn; ordinary joins pay the parent's posted credit fee. Each offer has
`current_fee: u64` and `increment: u64`, both of which may be zero.

The host supplies each lane's cumulative earnings as a precise fixed-point
number. The lane consumes only the increase since its previous checkpoint and
distributes it using the old eligible balances before a balance change. Totals
cannot decrease, and empty lanes consume their allocation without awarding
historical rewards to later entrants. Registration establishes a starting total;
earlier earnings are excluded.

The kernel has no gauges, votes, schedules, clocks, or emitter dependency. Its
eligible balances can be internal positions; token transfers are not required.
The host authenticates earnings and balances, binds each source budget to its
lane without duplication, and brings every affected source current before
changing balances or family terms. For a family change, every source total must
describe the same eligibility boundary. Monotonic totals alone cannot establish
that they are fresh. Custody, authorization, account layouts, and execution limits
also belong to the host; orchestration is outside this crate.

[Source documentation](src/lib.rs) explains accounting, rounding ownership,
resource bounds, and host duties. The [public actions](src/program.rs) are the
implementation interface. [SPEC.md](SPEC.md) defines the economic contract and
distinguishes referral accounting from the external allocator's responsibilities.

```bash
cargo doc -p mayf-referral --open
cargo run -p mayf-referral --example emissions
```

The example walks through an auction award, free ordinary joins, two asset lanes,
credit escrow, override-only marketing, and net collection. It supplies 200 and
100 credits directly to the two lanes and distributes them as 110, 107.5, and
82.5 credits to Alice, Bob, and Carol.
