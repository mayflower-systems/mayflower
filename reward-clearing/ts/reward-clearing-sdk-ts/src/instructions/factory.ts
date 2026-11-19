import type { PublicKey } from "@solana/web3.js"
import {
  addToRewardPotInstruction,
  type AddToRewardPotParams,
} from "./addToRewardPot.ts"
import {
  claimRewardsInstruction,
  type ClaimRewardsParams,
} from "./claimRewards.ts"
import {
  createBidderPositionInstruction,
  type CreateBidderPositionParams,
} from "./createBidderPosition.ts"
import {
  createChannelInstruction,
  type CreateChannelParams,
} from "./createChannel.ts"
import {
  createChannelEpochInstruction,
  type CreateChannelEpochParams,
} from "./createChannelEpoch.ts"
import {
  createChannelGroupInstruction,
  type CreateChannelGroupParams,
} from "./createChannelGroup.ts"
import {
  createEpochInstruction,
  type CreateEpochParams,
} from "./createEpoch.ts"
import {
  createOrganizationInstruction,
  type CreateOrganizationParams,
} from "./createOrganization.ts"
import {
  depositCreditsInstruction,
  type DepositCreditsParams,
} from "./depositCredits.ts"
import {
  finalizeChannelEpochInstruction,
  type FinalizeChannelEpochParams,
} from "./finalizeChannelEpoch.ts"
import {
  setChannelDefaultEpochTermsInstruction,
  type SetChannelDefaultEpochTermsParams,
} from "./setChannelDefaultEpochTerms.ts"
import {
  setChannelDisabledInstruction,
  type SetChannelDisabledParams,
} from "./setChannelDisabled.ts"
import {
  withdrawCreditsInstruction,
  type WithdrawCreditsParams,
} from "./withdrawCredits.ts"

/** Binds the program address once; each method takes only its instruction params. */
export const createRewardClearingInstructions = (programId: PublicKey) => ({
  addToRewardPot: (params: AddToRewardPotParams) =>
    addToRewardPotInstruction(programId, params),
  claimRewards: (params: ClaimRewardsParams) =>
    claimRewardsInstruction(programId, params),
  createBidderPosition: (params: CreateBidderPositionParams) =>
    createBidderPositionInstruction(programId, params),
  createChannel: (params: CreateChannelParams) =>
    createChannelInstruction(programId, params),
  createChannelEpoch: (params: CreateChannelEpochParams) =>
    createChannelEpochInstruction(programId, params),
  createChannelGroup: (params: CreateChannelGroupParams) =>
    createChannelGroupInstruction(programId, params),
  createEpoch: (params: CreateEpochParams) =>
    createEpochInstruction(programId, params),
  createOrganization: (params: CreateOrganizationParams) =>
    createOrganizationInstruction(programId, params),
  depositCredits: (params: DepositCreditsParams) =>
    depositCreditsInstruction(programId, params),
  finalizeChannelEpoch: (params: FinalizeChannelEpochParams) =>
    finalizeChannelEpochInstruction(programId, params),
  setChannelDefaultEpochTerms: (params: SetChannelDefaultEpochTermsParams) =>
    setChannelDefaultEpochTermsInstruction(programId, params),
  setChannelDisabled: (params: SetChannelDisabledParams) =>
    setChannelDisabledInstruction(programId, params),
  withdrawCredits: (params: WithdrawCreditsParams) =>
    withdrawCreditsInstruction(programId, params),
})

export type RewardClearingInstructions = ReturnType<
  typeof createRewardClearingInstructions
>
