import { PublicKey } from "@solana/web3.js"

export const BidderPositionSeed = "bidder_position"
export const OrganizationSeed = "organization"
export const ChannelGroupSeed = "channel_group"
export const ClearingTokenEscrowSeed = "clearing_token_escrow"
export const ChannelSeed = "reward_channel"
export const EpochSeed = "epoch"
export const ChannelEpochSeed = "channel_epoch"
export const ChannelEpochRewardPotEscrowSeed = "channel_epoch_reward_pot_escrow"
export const ChannelEpochClearingCreditsEscrowSeed =
  "channel_epoch_credits_escrow"
export const RewardStagingSeed = "reward_staging_token_escrow"

export interface RewardClearingPda {
  readonly programId: PublicKey

  organization(authority: PublicKey): PublicKey

  channelGroup(
    organization: PublicKey,
    nextChannelGroupIndex: number,
  ): PublicKey

  epoch(channelGroup: PublicKey, currentEpochIndex: number): PublicKey

  /** Escrow account that holds deposited clearing credits */
  clearingTokenEscrow(channelGroup: PublicKey): PublicKey

  channel(channelGroup: PublicKey, rewardTokenMint: PublicKey): PublicKey

  /** Channel-owned staging vault, shared across its epochs. */
  rewardStagingTokenAccount(channel: PublicKey): PublicKey

  channelEpoch(channel: PublicKey, epochCounter: number): PublicKey

  channelEpochRewardPotEscrow(
    channel: PublicKey,
    epochCounter: number,
  ): PublicKey

  channelEpochClearingCreditsEscrow(
    channel: PublicKey,
    epochCounter: number,
  ): PublicKey

  bidderPosition(
    channel: PublicKey,
    epochCounter: number,
    authority: PublicKey,
  ): PublicKey
}

/** Encodes the program's u32 indices without truncating or wrapping them. */
const indexSeed = (value: number): Uint8Array => {
  if (!Number.isInteger(value) || value < 0 || value > 0xffff_ffff) {
    throw new RangeError("PDA index must be an integer from 0 to 4294967295")
  }
  const bytes = new Uint8Array(4)
  new DataView(bytes.buffer).setUint32(0, value, true)
  return bytes
}

const encoder = new TextEncoder()

/** Derives addresses locally, using the program's exact seed order. */
export const createRewardClearingPda = (
  programId: PublicKey,
): RewardClearingPda => {
  const programAddress = (seed: string, ...seeds: Uint8Array[]) =>
    PublicKey.findProgramAddressSync(
      [encoder.encode(seed), ...seeds],
      programId,
    )[0]

  return {
    programId,
    organization: (authority) =>
      programAddress(OrganizationSeed, authority.toBytes()),
    channelGroup: (organization, nextChannelGroupIndex) =>
      programAddress(
        ChannelGroupSeed,
        organization.toBytes(),
        indexSeed(nextChannelGroupIndex),
      ),
    epoch: (channelGroup, currentEpochIndex) =>
      programAddress(
        EpochSeed,
        channelGroup.toBytes(),
        indexSeed(currentEpochIndex),
      ),
    clearingTokenEscrow: (channelGroup) =>
      programAddress(ClearingTokenEscrowSeed, channelGroup.toBytes()),
    channel: (channelGroup, rewardTokenMint) =>
      programAddress(
        ChannelSeed,
        channelGroup.toBytes(),
        rewardTokenMint.toBytes(),
      ),
    rewardStagingTokenAccount: (channel) =>
      programAddress(RewardStagingSeed, channel.toBytes()),
    channelEpoch: (channel, epochCounter) =>
      programAddress(
        ChannelEpochSeed,
        channel.toBytes(),
        indexSeed(epochCounter),
      ),
    channelEpochRewardPotEscrow: (channel, epochCounter) =>
      programAddress(
        ChannelEpochRewardPotEscrowSeed,
        channel.toBytes(),
        indexSeed(epochCounter),
      ),
    channelEpochClearingCreditsEscrow: (channel, epochCounter) =>
      programAddress(
        ChannelEpochClearingCreditsEscrowSeed,
        channel.toBytes(),
        indexSeed(epochCounter),
      ),
    bidderPosition: (channel, epochCounter, authority) =>
      programAddress(
        BidderPositionSeed,
        channel.toBytes(),
        indexSeed(epochCounter),
        authority.toBytes(),
      ),
  }
}
