// Generated from an Anchor IDL. Do not edit.

import { deserialize, serialize, type Schema } from "borsh"
import bs58 from "bs58"
import { PublicKey } from "@solana/web3.js"

export type PublicKeyBytes = ReadonlyArray<number>

export const publicKeyToBytes = (value: PublicKey): PublicKeyBytes =>
  Array.from(value.toBytes())

export const publicKeyFromBytes = (value: PublicKeyBytes): PublicKey =>
  new PublicKey(Uint8Array.from(value))

export const publicKeyBytesToBase58 = (value: PublicKeyBytes): string =>
  bs58.encode(Uint8Array.from(value))

export const hasDiscriminator = (
  data: Uint8Array,
  discriminator: Uint8Array,
): boolean =>
  data.length >= discriminator.length
  && discriminator.every((byte, index) => data[index] === byte)

export const decodeData = <T>(
  schema: Schema,
  discriminator: Uint8Array,
  data: Uint8Array,
): T => {
  if (!hasDiscriminator(data, discriminator)) {
    throw new Error("data does not match the expected discriminator")
  }
  return deserialize(schema, data.slice(discriminator.length)) as T
}

export const encodeData = (
  schema: Schema,
  discriminator: Uint8Array,
  value: unknown,
): Uint8Array => {
  const body = serialize(schema, value)
  const data = new Uint8Array(discriminator.length + body.length)
  data.set(discriminator)
  data.set(body, discriminator.length)
  return data
}
