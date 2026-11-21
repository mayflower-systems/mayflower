import {
  createEmitterInstruction,
  createWeightedEmitterInstructions,
  createWeightedEmitterPda,
  type BigDecimal,
} from "@mayflower-sys/weighted-emitter-sdk-ts"
import { Keypair } from "@solana/web3.js"
import { Bank, createUser, programId, setTime } from "./runtime.ts"
import { createMint, createTokenAccount, mintTokens } from "./tokens.ts"

export const startTime = 1_800_000_000n
export const emissionRate: BigDecimal.BigDecimal = { value: 125n, decimals: 1 }
export const annualDecayRate: BigDecimal.BigDecimal = { value: 0n, decimals: 0 }
export const pda = createWeightedEmitterPda(programId)
export const instructions = createWeightedEmitterInstructions(programId)

/** Create every program account through the public SDK, with explicit signers. */
export const createFixture = () => {
  setTime(startTime)
  const bank = new Bank()
  const payer = createUser()
  const authority = createUser()
  const voter = createUser()
  const emitter = Keypair.generate()
  const gauge = Keypair.generate()
  const voteMintAuthority = Keypair.generate()
  const voteMint = createMint(bank, payer, voteMintAuthority.publicKey)
  const mintAuthority = pda.mintAuthority(emitter.publicKey)
  const emissionMint = createMint(bank, payer, mintAuthority)
  const voteSource = createTokenAccount(bank, payer, voteMint, voter.publicKey)
  const emissionDestination = createTokenAccount(
    bank,
    payer,
    emissionMint,
    voter.publicKey,
  )
  mintTokens(bank, voteMint, voteMintAuthority, voteSource, 1_000n)

  // Emitter and gauge are new keypair accounts, so both sign their creation.
  bank.send(
    [
      createEmitterInstruction(programId, {
        authority: authority.publicKey,
        payer: payer.publicKey,
        emitter: emitter.publicKey,
        voteMint,
        emissionMint,
        emissionRate,
        annualDecayRate,
      }),
      instructions.createGauge({
        authority: authority.publicKey,
        payer: payer.publicKey,
        emitter: emitter.publicKey,
        gauge: gauge.publicKey,
      }),
    ],
    [authority, payer, emitter, gauge],
  )

  const selectedPosition = {
    authority: voter.publicKey,
    emitter: emitter.publicKey,
    gauge: gauge.publicKey,
    voteMint,
  }
  bank.send(
    [
      instructions.createVotePosition({
        ...selectedPosition,
        payer: payer.publicKey,
      }),
    ],
    [voter, payer],
  )
  const votePosition = pda.votePosition(gauge.publicKey, voter.publicKey)
  return {
    bank,
    payer,
    authority,
    voter,
    emitter: emitter.publicKey,
    gauge: gauge.publicKey,
    voteMint,
    emissionMint,
    mintAuthority,
    voteSource,
    emissionDestination,
    selectedPosition,
    votePosition,
    voteEscrow: pda.voteEscrow(votePosition),
  }
}
