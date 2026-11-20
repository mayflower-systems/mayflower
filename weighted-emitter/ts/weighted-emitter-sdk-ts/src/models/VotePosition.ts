import type { VotePosition as VotePositionAccount } from "@mayflower-sys/weighted-emitter-sol-client"
import * as Address from "./Address.ts"
import type { TokenBalance } from "./TokenBalance.ts"

/** The recorded vote principal that one authority holds in one gauge. */
export interface VotePosition {
  readonly authority: Address.Address
  readonly gauge: Address.Address
  readonly voteEscrow: Address.Address
  /** External token donations do not increase this position's voting weight. */
  readonly deposited: TokenBalance
  readonly selfBump: number
}

export const fromAccount = (x: VotePositionAccount): VotePosition => ({
  authority: Address.fromBytes(x.authority),
  gauge: Address.fromBytes(x.gauge),
  voteEscrow: Address.fromBytes(x.voteEscrow),
  deposited: { amount: x.deposited },
  selfBump: x.selfBump[0]!,
})

export const toAccount = (x: VotePosition): VotePositionAccount => ({
  authority: Address.toBytes(x.authority),
  gauge: Address.toBytes(x.gauge),
  voteEscrow: Address.toBytes(x.voteEscrow),
  deposited: x.deposited.amount,
  selfBump: [x.selfBump],
})
