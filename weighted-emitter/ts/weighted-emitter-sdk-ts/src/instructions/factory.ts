import type { PublicKey } from "@solana/web3.js"
import {
  createEmitterInstruction,
  type CreateEmitterParams,
} from "./createEmitter.ts"
import {
  createGaugeInstruction,
  type CreateGaugeParams,
} from "./createGauge.ts"
import {
  createVotePositionInstruction,
  type CreateVotePositionParams,
} from "./createVotePosition.ts"
import {
  depositVoteInstruction,
  type DepositVoteParams,
} from "./depositVote.ts"
import {
  setEmitterDecayRateInstruction,
  type SetEmitterDecayRateParams,
} from "./setEmitterDecayRate.ts"
import {
  settleEmissionInstruction,
  type SettleEmissionParams,
} from "./settleEmission.ts"
import {
  withdrawVoteInstruction,
  type WithdrawVoteParams,
} from "./withdrawVote.ts"

/** Binds the program address once; each method takes only its instruction params. */
export const createWeightedEmitterInstructions = (programId: PublicKey) => ({
  createEmitter: (params: CreateEmitterParams) =>
    createEmitterInstruction(programId, params),
  createGauge: (params: CreateGaugeParams) =>
    createGaugeInstruction(programId, params),
  createVotePosition: (params: CreateVotePositionParams) =>
    createVotePositionInstruction(programId, params),
  depositVote: (params: DepositVoteParams) =>
    depositVoteInstruction(programId, params),
  setEmitterDecayRate: (params: SetEmitterDecayRateParams) =>
    setEmitterDecayRateInstruction(programId, params),
  settleEmission: (params: SettleEmissionParams) =>
    settleEmissionInstruction(programId, params),
  withdrawVote: (params: WithdrawVoteParams) =>
    withdrawVoteInstruction(programId, params),
})

export type WeightedEmitterInstructions = ReturnType<
  typeof createWeightedEmitterInstructions
>
