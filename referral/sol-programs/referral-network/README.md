# Referral network on Solana

Anchor scaffold for the [referral accounting library](../../rust/mayf-referral).
The program follows reward-clearing's separate instruction handlers, account
layouts, seed constants and domain-error mapping.

`create_network` validates immutable tree terms and creates a network bound to
its authority and an existing SPL Token credit mint. The other ten entry points
have typed primary accounts and arguments and return `InstructionNotImplemented`.
Their token, auction and earnings integrations are still to be wired.

[Entry points](prog/src/lib.rs) · [Account layouts](prog/src/state.rs) ·
[Instruction contracts and remaining work](prog/src/instructions/spec.md)

From the workspace root:

```bash
cargo check --locked -p mayf-referral-network-sol
cargo clippy --locked -p mayf-referral-network-sol --all-targets -- -D warnings
cargo test --locked -p mayf-referral-network-sol -p mayf-referral
```

The existing program address is a development placeholder. Deployment artifacts
and generated clients are not included in this scaffold.
