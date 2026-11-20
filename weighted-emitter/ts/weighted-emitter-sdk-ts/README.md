# @mayflower-sys/weighted-emitter-sdk-ts

Plain account models, PDA helpers, and instruction builders for the Weighted Emitter program. The SDK wraps `@mayflower-sys/weighted-emitter-sol-client`; all conversions and builders run locally without RPC calls.

## Reading accounts

Decode account data with the generated client, then convert it into a model:

```ts
import { decodeEmitterAccount } from "@mayflower-sys/weighted-emitter-sol-client"
import { DecimalNumber } from "@mayflower-sys/fixed-number-ts"
import { Emitter } from "@mayflower-sys/weighted-emitter-sdk-ts"

// `data` is the account data returned by an RPC node.
const emitter = Emitter.fromAccount(decodeEmitterAccount(data))

emitter.totalDeposited.amount // bigint vote token base units
emitter.lastUpdatedAtUnixSeconds.seconds // bigint Unix seconds
DecimalNumber.format(emitter.emissionRate) // exact decimal text
```

`Emitter`, `Gauge`, and `VotePosition` each export their model type and `fromAccount` / `toAccount` functions. Models are readonly plain objects. `Address` wraps `{ bytes: Uint8Array }`, `TokenBalance` wraps `{ amount: bigint }`, and `UnixTimestamp` wraps `{ seconds: bigint }`, following the reward-clearing SDK's structural branding pattern.

`BigDecimal` is `{ value: bigint, decimals: number }`, meaning `value / 10^decimals`. Both Q64.64 rates and Q192.64 indexes read exactly, with 64 decimal places. Index conversion handles the program's 32 little-endian bytes through the existing fixed-number driver.

`Emitter.toAccount` and `Gauge.toAccount` return the fixed-number package's `Result`: `{ ok: true, value }` or `{ ok: false, error }`. They reject decimal values that are negative, overflow, or do not fit their stored grid exactly. `VotePosition.toAccount` returns the generated account directly. Account conversions check fixed-point representation; program rules remain with the program.

## Deriving addresses

```ts
import { PublicKey } from "@solana/web3.js"
import { createWeightedEmitterPda } from "@mayflower-sys/weighted-emitter-sdk-ts"

const programId = new PublicKey("6tdyEnBkiNYwbF5FmiF4Dq8piqq16CA642JEkMS88RgL")
const pda = createWeightedEmitterPda(programId)

const mintAuthority = pda.mintAuthority(emitterAddress)
const votePosition = pda.votePosition(gaugeAddress, authority)
const voteEscrow = pda.voteEscrow(votePosition)
```

Emitter and gauge accounts are created from new keypairs. Their addresses are explicit builder inputs, and their keypairs must sign account creation. Before creating an emitter, initialize its emission mint with `pda.mintAuthority(emitterAddress)` as the mint authority.

## Building instructions

Every instruction has a builder that takes `(programId, params)` and returns a web3.js `TransactionInstruction`. `createWeightedEmitterInstructions(programId)` binds the program ID once and exposes the same builders without the `Instruction` suffix:

```ts
import { createWeightedEmitterInstructions } from "@mayflower-sys/weighted-emitter-sdk-ts"

const builders = createWeightedEmitterInstructions(programId)
const position = {
  authority,
  emitter: emitterAddress,
  gauge: gaugeAddress,
  voteMint,
}
const instructions = [
  builders.createVotePosition({ ...position, payer: authority }),
  builders.depositVote({
    ...position,
    voteSource,
    amount: 1_000n,
  }),
]
// Add these to a web3.js Transaction, then sign and submit it in your app.
```

The factory also exposes `createEmitter`, `createGauge`, `withdrawVote`, `setEmitterDecayRate`, and `settleEmission`. Builders derive mint authorities, vote positions, and escrows as needed. The generated client supplies account order, signer and writable flags, fixed program accounts, and instruction encoding. Token source and destination accounts stay explicit.

Creation and decay updates accept `BigDecimal` rates:

```ts
const instruction = builders.createEmitter({
  authority,
  payer: authority,
  emitter: emitterAddress,
  voteMint,
  emissionMint,
  emissionRate: { value: 1_000n, decimals: 0 },
  annualDecayRate: { value: 125n, decimals: 3 },
})
```

Rates must be exactly representable as unsigned Q64.64; 0.125 is exact, while 0.1 is not. To choose rounding explicitly, use `U64F64.fromDecimalNumber(value, "floor")` or `"ceil"` from `@mayflower-sys/fixed-number-ts`, then `BigDecimal.fromQ64f64` on the successful result. Builders throw `RangeError` for unrepresentable rates or amounts outside `u64`. Amounts use token base units as `bigint`. Building an instruction does not sign or submit it.

## Building the package

From the workspace root:

```sh
pnpm --filter @mayflower-sys/weighted-emitter-sdk-ts... build
```
