# Weighted Emitter SDK runtime examples

Small LiteSVM integration tests for the public `@mayflower-sys/weighted-emitter-sdk-ts` API. They demonstrate all seven instruction builders, SDK-derived addresses, explicit signers, and reading real program accounts through generated decoders into SDK models. The cases cover creation, voting, a decimal decay update, and emission settlement. Program mathematics, token custody, and authorization scenarios stay in the Rust suites.

From the repository root, with the workspace's Node 26 toolchain and dependencies installed:

```sh
pnpm --filter @mayflower-sys/weighted-emitter-sdk-litesvm-tests... build
pnpm --filter @mayflower-sys/weighted-emitter-sdk-litesvm-tests test
```

The suite loads `weighted-emitter/sol-programs/weighted-emitter/bin/mayf_weighted_emitter_sol.so` relative to the test module, independently of the working directory. Missing or invalid program artifacts fail the suite. Run `weighted-emitter/sol-programs/weighted-emitter/build.sh` after changing the program to refresh the artifact and generated client.

Start with [`test/sdk.test.ts`](test/sdk.test.ts) for client examples. [`test/prelude/fixture.ts`](test/prelude/fixture.ts) shows mint setup with the SDK-derived mint authority, keypair account creation, and position creation. [`test/prelude/accounts.ts`](test/prelude/accounts.ts) shows the account read boundary: account bytes, generated decoder, then SDK `fromAccount`.

The tests use the same pinned LiteSVM 0.2.0 and web3.js v1 setup as the reward-clearing SDK suite. One VM runs the sequential tests, and each case creates fresh accounts and sets its clock explicitly. All mints, token accounts, and program accounts are created through real instructions. Turbo test caching is disabled, and the TypeScript CI job runs the suite after the workspace build.
