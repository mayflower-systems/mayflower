// Generated from an Anchor IDL. Do not edit.
//
// The Solana types this client uses, so that it does not depend on
// @solana/web3.js.

import bs58 from "bs58"
import type { PublicKeyBytes } from "./codecs.js"

/** A Solana address: the 32 bytes of a public key or a program derived address. */
export type PublicKey = Uint8Array

/** Copies 32 bytes, such as a model's `PublicKeyBytes`, into a new public key. */
export const publicKeyFromBytes = (value: ArrayLike<number>): PublicKey => {
  if (value.length !== 32) {
    throw new RangeError("a public key must be 32 bytes")
  }
  return Uint8Array.from(value)
}

/** Copies a public key into the plain numbers that Borsh models hold. */
export const publicKeyToBytes = (value: PublicKey): PublicKeyBytes =>
  Array.from(value)

export const publicKeyToBase58 = (value: PublicKey | PublicKeyBytes): string =>
  bs58.encode(Uint8Array.from(value))

export const publicKeyFromBase58 = (value: string): PublicKey =>
  publicKeyFromBytes(bs58.decode(value))

/** One account an instruction uses, with the access the program requires. */
export interface AccountMeta {
  pubkey: PublicKey
  isSigner: boolean
  isWritable: boolean
}

/** The program to run, its accounts in order, and the encoded instruction data. */
export interface TransactionInstruction {
  programId: PublicKey
  keys: AccountMeta[]
  data: Uint8Array
}
