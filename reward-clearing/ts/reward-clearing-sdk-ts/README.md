# @mayflower-sys/reward-clearing-sdk-ts

Models of the Reward Clearing program's accounts, for application code. Every model is plain data, and every function is pure.

The generated client `@mayflower-sys/reward-clearing-sol-client` decodes an account into the types of the Anchor IDL. There, a public key is an array of 32 numbers and a fraction is a raw `u128`. This package converts those types into models that are easier to read and to compute with.

| In the generated client                                       | In a model                                          |
| ------------------------------------------------------------- | --------------------------------------------------- |
| `PublicKeyBytes`                                              | `Address`: `{ bytes: Uint8Array }`                  |
| a `u64` amount of tokens                                      | `TokenBalance`: `{ amount: bigint }`                |
| a `u64` time                                                  | `UnixTimestamp`: `{ seconds: bigint }`              |
| a fraction: a Q64.64 `u128`, or a `u32` of micro-basis points | `BigDecimal`: `{ value: bigint, decimals: number }` |
| a bump seed, `[u8; 1]`                                        | `number`                                            |

The wrapper types keep different kinds of value apart. A `TokenBalance` and a `UnixTimestamp` both hold a `bigint`, but the compiler rejects one where the other belongs.

## Reading an account

```ts
import {
  decodeChannelEpochAccount,
  publicKeyToBase58,
} from "@mayflower-sys/reward-clearing-sol-client"
import { ChannelEpoch } from "@mayflower-sys/reward-clearing-sdk-ts"

// `data` is the account data that an RPC node returned.
const channelEpoch: ChannelEpoch.ChannelEpoch = ChannelEpoch.fromAccount(
  decodeChannelEpochAccount(data),
)

channelEpoch.rewardPotBalance.amount // bigint
channelEpoch.terms.rampBonusSlope // BigDecimal
publicKeyToBase58(channelEpoch.channel.bytes) // string
```

Each module exports one model type, named as the module, and the plain functions that convert it.

| Module            | Generated type                      |
| ----------------- | ----------------------------------- |
| `Organization`    | `Organization`                      |
| `ChannelGroup`    | `ChannelGroup`                      |
| `Epoch`           | `Epoch`                             |
| `Channel`         | `Channel`                           |
| `ChannelEpoch`    | `ChannelEpoch`                      |
| `BidderPosition`  | `BidderPosition`                    |
| `BidderLot`       | `BidderLot`                         |
| `FinalizedTotals` | `StoredFinalizedTotals`             |
| `EpochTerms`      | `EpochTerms` and `StoredEpochTerms` |

The program stores epoch terms in two formats, and both read as the one model `EpochTerms`:

- `EpochTerms.fromAccount` reads the default terms of a channel, which hold the slope in micro-basis points.
- `EpochTerms.fromStored` reads the terms of a channel epoch, which hold the slope as Q64.64.

## Numbers

A `BigDecimal` is exact: `{ value: 123n, decimals: 2 }` is 1.23. Reading an account never rounds.

A `BigDecimal` has the shape of `DecimalNumber` in `@mayflower-sys/fixed-number-ts`, so the functions of that package accept it:

```ts
import { DecimalNumber } from "@mayflower-sys/fixed-number-ts"

DecimalNumber.format(channelEpoch.terms.rampBonusSlope) // "1.5"
```

A Q64.64 value reads with 64 decimals. The program rounds a channel's slope down to Q64.64 when it creates a channel epoch. So a default slope of 0.1 reads back from the channel epoch as 0.09999999999999999996747…, the nearest Q64.64 value below 0.1.

## Writing an account

`toAccount` is the inverse of `fromAccount`: a model that `fromAccount` returned converts back to the same account.

A decimal number may not fit a stored format. 0.1 is not a Q64.64 value, and 0.0000001 is not a whole count of micro-basis points. So `toAccount` returns a `Result` for every model that holds a `BigDecimal`:

```ts
import { EpochTerms } from "@mayflower-sys/reward-clearing-sdk-ts"

const terms = EpochTerms.toAccount({
  rampBonusSlope: { value: 15n, decimals: 1 },
  exitPenaltyExponent: 2,
})
if (terms.ok) {
  terms.value // { rampBonusSlopeMbps: 1_500_000, exitPenaltyExponent: 2 }
} else {
  terms.error // "Inexact", "Overflow" or "NegativeToUnsigned"
}
```

`toAccount` does not round. It checks that each number fits its stored format, and nothing else: the program's rules, such as the range of a slope, stay with the program.

## Tests

```sh
pnpm test
```
