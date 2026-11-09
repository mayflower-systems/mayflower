// Generated from an Anchor IDL. Do not edit.

import { deserialize, serialize, type Schema } from "borsh"

/**
 * A public key as Borsh models hold it: 32 numbers, one per byte. `solana.ts`
 * converts it to and from `PublicKey`.
 */
export type PublicKeyBytes = ReadonlyArray<number>

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
