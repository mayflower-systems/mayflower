# mayf-referral

A pure state machine for fixed upline commissions, credit-escrow family marketing,
and proportional lane allocation. Public actions validate their inputs and return
next state plus custody effects. The host applies each receipt atomically.

The host configures maximum depth `H` and commission `q`, subject to `H*q < 1`.
Both zero depth and zero commission are valid. The current lane mask has storage
capacity for 16 assets; tree height is a separate configuration. Asset changes
touch one ancestor path; family changes touch one parent's source lanes. No
action loads a subtree. Root seats consume external auction awards and return a
credit burn; ordinary joins pay the parent's posted credit fee. Each offer has
`current_fee: u64` and `increment: u64`, both of which may be zero.

The host supplies the authenticated cumulative emission integral, allowing the
planned continuously declining rate without coupling the kernel to a particular
curve or global emission parameters. The host also owns token custody,
authorization, account layouts, and execution limits for its chosen depth.

[Source documentation](src/lib.rs) explains accounting, rounding ownership,
resource bounds, and host duties. The [public actions](src/program.rs) are the
implementation interface. [SPEC.md](SPEC.md) defines the economic contract;
[MIGRATION.md](MIGRATION.md) records the architectural review and migration.

```bash
cargo doc -p mayf-referral --open
cargo run -p mayf-referral --example emissions
```

The example walks through an auction award, free ordinary joins, two asset lanes,
credit escrow, override-only marketing, and net collection. It allocates 300
credits as 110, 107.5, and 82.5 credits to Alice, Bob, and Carol.
