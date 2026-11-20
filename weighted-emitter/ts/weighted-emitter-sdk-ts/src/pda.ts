import { PublicKey } from "@solana/web3.js"

export const MintAuthoritySeed = "mint_authority"
export const VotePositionSeed = "vote_position"
export const VoteEscrowSeed = "vote_escrow"

export interface WeightedEmitterPda {
  readonly programId: PublicKey

  /** Authority of an emitter's emission mint. */
  mintAuthority(emitter: PublicKey): PublicKey

  /** One position per gauge and voting authority. */
  votePosition(gauge: PublicKey, authority: PublicKey): PublicKey

  /** Token escrow owned by a vote position. */
  voteEscrow(votePosition: PublicKey): PublicKey
}

const encoder = new TextEncoder()

/**
 * Derives the program's PDAs locally, using the exact seed order.
 * Emitter and gauge accounts are keypair accounts, not PDAs.
 */
export const createWeightedEmitterPda = (
  programId: PublicKey,
): WeightedEmitterPda => {
  const programAddress = (seed: string, ...seeds: Uint8Array[]) =>
    PublicKey.findProgramAddressSync(
      [encoder.encode(seed), ...seeds],
      programId,
    )[0]

  return {
    programId,
    mintAuthority: (emitter) =>
      programAddress(MintAuthoritySeed, emitter.toBytes()),
    votePosition: (gauge, authority) =>
      programAddress(VotePositionSeed, gauge.toBytes(), authority.toBytes()),
    voteEscrow: (votePosition) =>
      programAddress(VoteEscrowSeed, votePosition.toBytes()),
  }
}
