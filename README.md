# Mayflower

Mayflower is a systemic library for non-custodial & decentralized incentive games on public blockchains.

The [Amplifying Value Machine](./avm/rust/mayf-avm-prog/README.md) implements a deterministic issuance schedule for claims on a reserve, while preserving a never-decreasing minimum redemption value.

The [Reward Distributor](./reward-clearing/rust/mayf-reward-clearing/README.md) uses a two-way time-conditioned bidding mechanism for hosting fair auctions on public blockchains.

## Getting acquainted

Explore the [wiki](https://github.com/mayflower-system/mayflower/wiki) for a specification and overview of the modules.

Engage in [discussions](https://github.com/mayflower-system/mayflower/discussions). Announcements will be posted here.

## Talk to maintainers

Inquiries can be sent to contact@mayflower.systems

## Testing

The mono-repo is primarily written in Rust and Typescript.

To verify tests, run:

```bash
cargo test
pnpm test
```

## Status

Currently incomplete and unstable.  Release date TBD.

## License

Apache 2.0
