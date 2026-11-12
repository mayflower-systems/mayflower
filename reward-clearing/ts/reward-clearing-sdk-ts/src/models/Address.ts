import {
  publicKeyFromBytes,
  publicKeyToBytes,
  type PublicKeyBytes,
} from "@mayflower-sys/reward-clearing-sol-client"

/**
 * A Solana address: the 32 bytes of a public key or a program derived
 * address.
 *
 * `publicKeyToBase58(address.bytes)` from the generated client gives the text
 * form.
 */
export interface Address {
  readonly bytes: Uint8Array
}

/** Reads an address from the 32 numbers that a generated model holds. */
export const fromBytes = (x: PublicKeyBytes): Address => ({
  bytes: publicKeyFromBytes(x),
})

/** Writes an address as the 32 numbers that a generated model holds. */
export const toBytes = (x: Address): PublicKeyBytes => publicKeyToBytes(x.bytes)
