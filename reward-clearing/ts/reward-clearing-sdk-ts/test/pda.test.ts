import assert from "node:assert/strict"
import { test } from "node:test"
import * as client from "@mayflower-sys/reward-clearing-sol-client"
import { PublicKey } from "@solana/web3.js"
import * as sdk from "../src/index.ts"
import {
  addresses,
  authority,
  channelGroupIndex,
  epochCounter,
  programId,
  rewardTokenMint,
} from "./addresses.ts"

const pda = sdk.createRewardClearingPda(programId)

const deriveAddresses = (pda: sdk.RewardClearingPda) => {
  const organization = pda.organization(authority)
  const channelGroup = pda.channelGroup(organization, channelGroupIndex)
  const channel = pda.channel(channelGroup, rewardTokenMint)
  return {
    organization,
    channelGroup,
    clearingTokenEscrow: pda.clearingTokenEscrow(channelGroup),
    channel,
    rewardStagingTokenAccount: pda.rewardStagingTokenAccount(channel),
    initialEpoch: pda.epoch(channelGroup, 0),
    epoch: pda.epoch(channelGroup, epochCounter),
    nextEpoch: pda.epoch(channelGroup, epochCounter + 1),
    channelEpoch: pda.channelEpoch(channel, epochCounter),
    rewardPotEscrow: pda.channelEpochRewardPotEscrow(channel, epochCounter),
    clearingCreditsEscrow: pda.channelEpochClearingCreditsEscrow(
      channel,
      epochCounter,
    ),
    bidderPosition: pda.bidderPosition(channel, epochCounter, authority),
  }
}

test("all PDA helpers match independent Rust address vectors", () => {
  const actual = deriveAddresses(pda)
  for (const name of Object.keys(addresses) as (keyof typeof addresses)[]) {
    assert.equal(actual[name].toBase58(), addresses[name].toBase58(), name)
  }
})

test("exported seed strings match the generated program constants", () => {
  const seeds = [
    [sdk.OrganizationSeed, client.ORGANIZATION_SEED],
    [sdk.ChannelGroupSeed, client.CHANNEL_GROUP_SEED],
    [sdk.ClearingTokenEscrowSeed, client.CLEARING_TOKEN_ESCROW_SEED],
    [sdk.ChannelSeed, client.REWARD_CHANNEL_SEED],
    [sdk.RewardStagingSeed, client.REWARD_STAGING_SEED],
    [sdk.EpochSeed, client.EPOCH_SEED],
    [sdk.ChannelEpochSeed, client.CHANNEL_EPOCH_SEED],
    [
      sdk.ChannelEpochRewardPotEscrowSeed,
      client.CHANNEL_EPOCH_REWARD_POT_ESCROW_SEED,
    ],
    [
      sdk.ChannelEpochClearingCreditsEscrowSeed,
      client.CHANNEL_EPOCH_CLEARING_CREDITS_ESCROW_SEED,
    ],
    [sdk.BidderPositionSeed, client.BIDDER_POSITION_SEED],
  ] as const
  for (const [text, bytes] of seeds) {
    assert.deepEqual(Array.from(new TextEncoder().encode(text)), bytes)
  }
})

test("the supplied program ID is used by every helper", () => {
  const other = deriveAddresses(sdk.createRewardClearingPda(PublicKey.default))
  for (const name of Object.keys(addresses) as (keyof typeof addresses)[]) {
    assert.notEqual(other[name].toBase58(), addresses[name].toBase58(), name)
  }
})

test("u32 seeds accept boundaries and reject values that would truncate or wrap", () => {
  const indexed = [
    (index: number) => pda.channelGroup(addresses.organization, index),
    (index: number) => pda.epoch(addresses.channelGroup, index),
    (index: number) => pda.channelEpoch(addresses.channel, index),
    (index: number) =>
      pda.channelEpochRewardPotEscrow(addresses.channel, index),
    (index: number) =>
      pda.channelEpochClearingCreditsEscrow(addresses.channel, index),
    (index: number) => pda.bidderPosition(addresses.channel, index, authority),
  ]
  for (const derive of indexed) {
    assert.doesNotThrow(() => derive(0))
    assert.doesNotThrow(() => derive(0xffff_ffff))
    for (const index of [-1, 0.5, 0x1_0000_0000, NaN, Infinity]) {
      assert.throws(() => derive(index), RangeError)
    }
  }
})
