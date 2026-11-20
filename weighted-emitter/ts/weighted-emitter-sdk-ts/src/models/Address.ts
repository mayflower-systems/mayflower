import {
  publicKeyFromBytes,
  publicKeyToBytes,
  type PublicKeyBytes,
} from "@mayflower-sys/weighted-emitter-sol-client"

/** A Solana address as plain data, independent of a wallet or RPC library. */
export interface Address {
  readonly bytes: Uint8Array
}

/** Copies the generated model's 32 numbers into an address. */
export const fromBytes = (x: PublicKeyBytes): Address => ({
  bytes: publicKeyFromBytes(x),
})

/** Copies an address into the numbers that a generated model holds. */
export const toBytes = (x: Address): PublicKeyBytes => publicKeyToBytes(x.bytes)
