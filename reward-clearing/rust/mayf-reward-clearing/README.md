# mayf-reward-clearing

The reward-clearing auction as pure Rust functions. Tests can run the clearing
logic without a Solana runtime.

The wiki's [Revenue Clearing](../../../docs/github-wiki/content/Revenue-Clearing/Revenue-Clearing.md)
pages specify the algorithm. The crate documentation explains how a host applies
the actions:

```bash
cargo doc -p mayf-reward-clearing --open
```

The auction example runs the wiki's worked example, from the first deposit to
the last claim:

```bash
cargo run -p mayf-reward-clearing --example auction
```

Run the tests from the repository root:

```bash
cargo test -p mayf-reward-clearing --all-targets
```
