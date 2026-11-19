import {
  createOrganizationInstruction,
  createRewardClearingInstructions,
  createRewardClearingPda,
  type EpochTerms,
} from "@mayflower-sys/reward-clearing-sdk-ts"
import { readChannelGroup, readOrganization } from "./accounts.ts"
import { Bank, createUser, programId, setTime } from "./runtime.ts"
import { createMint, createTokenAccount, mintTokens } from "./tokens.ts"

export const startTime = 1_800_000_000n
export const duration = 3_600
export const initialTokens = 10_000n
export const defaultTerms: EpochTerms.EpochTerms = {
  rampBonusSlope: { value: 0n, decimals: 0 },
  exitPenaltyExponent: 1,
}

export const pda = createRewardClearingPda(programId)
export const instructions = createRewardClearingInstructions(programId)

/** Fresh external accounts, before any program account has been created. */
export const createPrelude = () => {
  setTime(startTime)
  const bank = new Bank()
  const payer = createUser()
  const authority = createUser()
  const bidder = createUser()
  const creditsMint = createMint(bank, payer)
  const rewardMint = createMint(bank, payer)
  const creditsSource = createTokenAccount(
    bank,
    payer,
    creditsMint.address,
    bidder.publicKey,
  )
  const rewardSource = createTokenAccount(
    bank,
    payer,
    rewardMint.address,
    authority.publicKey,
  )
  const rewardDestination = createTokenAccount(
    bank,
    payer,
    rewardMint.address,
    bidder.publicKey,
  )
  mintTokens(bank, creditsMint, creditsSource, initialTokens)
  mintTokens(bank, rewardMint, rewardSource, initialTokens)
  const organization = pda.organization(authority.publicKey)
  return {
    bank,
    payer,
    authority,
    bidder,
    creditsMint,
    rewardMint,
    creditsSource,
    rewardSource,
    rewardDestination,
    organization,
  }
}

/** Normal SDK creation with a sponsor distinct from the authority and bidder. */
export const createFixture = () => {
  const f = createPrelude()
  f.bank.send(
    [
      createOrganizationInstruction(programId, {
        authority: f.authority.publicKey,
        payer: f.payer.publicKey,
      }),
    ],
    [f.authority, f.payer],
  )
  const { nextChannelGroupIndex } = readOrganization(f.organization)
  const channelGroup = pda.channelGroup(f.organization, nextChannelGroupIndex)
  f.bank.send(
    [
      instructions.createChannelGroup({
        authority: f.authority.publicKey,
        payer: f.payer.publicKey,
        nextChannelGroupIndex,
        clearingCreditsMint: f.creditsMint.address,
        masterStartTimeUnixSeconds: startTime,
        epochDurationSeconds: duration,
      }),
    ],
    [f.authority, f.payer],
  )
  const { currentEpochIndex } = readChannelGroup(channelGroup)
  const channel = pda.channel(channelGroup, f.rewardMint.address)
  f.bank.send(
    [
      instructions.createChannel({
        authority: f.authority.publicKey,
        payer: f.payer.publicKey,
        channelGroup,
        currentEpochIndex,
        rewardTokenMint: f.rewardMint.address,
        clearingCreditsMint: f.creditsMint.address,
        defaultEpochTerms: defaultTerms,
      }),
    ],
    [f.authority, f.payer],
  )
  const selectedEpoch = {
    channelGroup,
    channel,
    epochCounter: currentEpochIndex,
  }
  f.bank.send(
    [
      instructions.createBidderPosition({
        ...selectedEpoch,
        authority: f.bidder.publicKey,
        payer: f.payer.publicKey,
      }),
    ],
    [f.bidder, f.payer],
  )
  return {
    ...f,
    ...selectedEpoch,
    selectedEpoch,
    epoch: pda.epoch(channelGroup, currentEpochIndex),
    channelEpoch: pda.channelEpoch(channel, currentEpochIndex),
    position: pda.bidderPosition(
      channel,
      currentEpochIndex,
      f.bidder.publicKey,
    ),
    staging: pda.rewardStagingTokenAccount(channel),
    groupCredits: pda.clearingTokenEscrow(channelGroup),
    rewardPot: pda.channelEpochRewardPotEscrow(channel, currentEpochIndex),
    creditsEscrow: pda.channelEpochClearingCreditsEscrow(
      channel,
      currentEpochIndex,
    ),
  }
}
