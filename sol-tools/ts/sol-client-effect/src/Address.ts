import bs58 from "bs58"
import { Effect, Schema, SchemaGetter, SchemaIssue } from "effect"

/**
 * A Solana address: the 32 raw bytes of an ed25519 public key or a
 * program-derived address.
 */
export const Address = Schema.Uint8Array.check(
  Schema.isBetweenLength(32, 32),
).pipe(Schema.brand("Address"))

export type Address = typeof Address.Type

/** Encodes an address as a base58 string. */
export const toBase58 = (address: Address): string => bs58.encode(address)

/**
 * A codec between a base58 string and an `Address`. Decoding fails if the
 * string is not valid base58 or does not decode to exactly 32 bytes.
 */
export const AddressFromBase58 = Schema.String.annotate({
  expected: "a base58 encoded Solana address",
}).pipe(
  Schema.decodeTo(Address, {
    decode: SchemaGetter.transformEffect((input, options) => {
      const bytes = bs58.decodeUnsafe(input)
      return bytes === undefined ?
          Effect.fail(
            new SchemaIssue.InvalidValue(
              { expected: "a valid base58 string" },
              input,
              options,
            ),
          )
        : Effect.succeed(bytes)
    }),
    encode: SchemaGetter.transform((bytes) => bs58.encode(bytes)),
  }),
)

/** Decodes a base58 string into an `Address`. */
export const fromBase58: (
  input: string,
) => Effect.Effect<Address, Schema.SchemaError> =
  Schema.decodeEffect(AddressFromBase58)
