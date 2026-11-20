import assert from "node:assert/strict"
import {
  BidderPosition,
  Channel,
  ChannelEpoch,
  ChannelGroup,
  Epoch,
  Organization,
} from "@mayflower-sys/reward-clearing-sdk-ts"
import {
  decodeBidderPositionAccount,
  decodeChannelAccount,
  decodeChannelEpochAccount,
  decodeChannelGroupAccount,
  decodeEpochAccount,
  decodeOrganizationAccount,
} from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey } from "@solana/web3.js"
import { account, programId } from "./runtime.ts"

// This is the SDK's documented read boundary: RPC/LiteSVM bytes -> generated
// account decoder -> SDK model. No dependency on SDK internals or IDL fixtures.
const programData = (address: PublicKey): Uint8Array => {
  const info = account(address)
  assert(info.owner.equals(programId), `Wrong owner of ${address.toBase58()}`)
  return info.data
}

export const readOrganization = (address: PublicKey) =>
  Organization.fromAccount(decodeOrganizationAccount(programData(address)))

export const readChannelGroup = (address: PublicKey) =>
  ChannelGroup.fromAccount(decodeChannelGroupAccount(programData(address)))

export const readEpoch = (address: PublicKey) =>
  Epoch.fromAccount(decodeEpochAccount(programData(address)))

export const readChannel = (address: PublicKey) =>
  Channel.fromAccount(decodeChannelAccount(programData(address)))

export const readChannelEpoch = (address: PublicKey) =>
  ChannelEpoch.fromAccount(decodeChannelEpochAccount(programData(address)))

export const readBidderPosition = (address: PublicKey) =>
  BidderPosition.fromAccount(decodeBidderPositionAccount(programData(address)))
