import assert from "node:assert/strict"
import {
  Emitter,
  Gauge,
  VotePosition,
} from "@mayflower-sys/weighted-emitter-sdk-ts"
import {
  decodeEmitterAccount,
  decodeGaugeAccount,
  decodeVotePositionAccount,
} from "@mayflower-sys/weighted-emitter-sol-client"
import type { PublicKey } from "@solana/web3.js"
import { account, programId } from "./runtime.ts"

// Application read boundary: account bytes -> generated decoder -> SDK model.
const programData = (address: PublicKey): Uint8Array => {
  const info = account(address)
  assert(info.owner.equals(programId), `Wrong owner of ${address.toBase58()}`)
  return info.data
}

export const readEmitter = (address: PublicKey) =>
  Emitter.fromAccount(decodeEmitterAccount(programData(address)))

export const readGauge = (address: PublicKey) =>
  Gauge.fromAccount(decodeGaugeAccount(programData(address)))

export const readVotePosition = (address: PublicKey) =>
  VotePosition.fromAccount(decodeVotePositionAccount(programData(address)))
