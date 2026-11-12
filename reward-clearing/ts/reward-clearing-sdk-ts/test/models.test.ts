import assert from "node:assert/strict"
import { test } from "node:test"
import { DecimalNumber, err, ok } from "@mayflower-sys/fixed-number-ts"
import type * as client from "@mayflower-sys/reward-clearing-sol-client"
import {
  BidderPosition,
  Channel,
  ChannelEpoch,
  ChannelGroup,
  Epoch,
  EpochTerms,
  Organization,
} from "../src/index.ts"

/** A stand-in address: 32 copies of one byte. */
const address = (byte: number): client.PublicKeyBytes =>
  Array.from({ length: 32 }, () => byte)

/** The raw Q64.64 value of 0.5. */
const HALF = 1n << 63n

const organization: client.Organization = {
  authority: address(1),
  nextChannelGroupIndex: 3,
}

const channelGroup: client.ChannelGroup = {
  organization: address(1),
  channelGroupIndex: 2,
  clearingCreditsMint: address(3),
  masterStartTimeUnixSeconds: 1_700_000_000n,
  epochDurationSeconds: 604_800,
  currentEpochIndex: 9,
  selfBump: [255],
}

const epoch: client.Epoch = {
  channelGroup: address(1),
  epochCounter: 9,
  startTimeUnixSeconds: 1_705_443_200n,
  durationSeconds: 604_800,
}

const channel: client.Channel = {
  channelGroup: address(1),
  rewardTokenMint: address(2),
  rewardStagingTokenAccount: address(3),
  defaultEpochTerms: { rampBonusSlopeMbps: 1_500_000, exitPenaltyExponent: 2 },
  lastCreatedEpochCounter: 9,
  disabled: true,
  selfBump: [254],
}

const channelEpoch: client.ChannelEpoch = {
  channel: address(1),
  epochCounter: 9,
  rewardPotEscrowTokenAccount: address(2),
  clearingCreditsEscrowTokenAccount: address(3),
  clearingCreditsBalance: 1_000n,
  rewardPotBalance: 250n,
  weightedSharesBalanceQ64f64: 5n * HALF,
  terms: { rampBonusSlopeQ64f64: 3n * HALF, exitPenaltyExponent: 2 },
  finalTotals: { rewardPot: 300n, weightedSharesQ64f64: 7n * HALF },
  selfBump: [253],
}

const bidderPosition: client.BidderPosition = {
  authority: address(1),
  channel: address(2),
  epochCounter: 9,
  claimed: false,
  selfBump: [252],
  lots: [
    { credits: 600n, sharesQ64f64: 3n * HALF },
    { credits: 400n, sharesQ64f64: HALF },
  ],
}

test("a model writes back the account it was read from", () => {
  assert.deepStrictEqual(
    Organization.toAccount(Organization.fromAccount(organization)),
    organization,
  )
  assert.deepStrictEqual(
    ChannelGroup.toAccount(ChannelGroup.fromAccount(channelGroup)),
    channelGroup,
  )
  assert.deepStrictEqual(Epoch.toAccount(Epoch.fromAccount(epoch)), epoch)
  assert.deepStrictEqual(
    Channel.toAccount(Channel.fromAccount(channel)),
    ok(channel),
  )
  assert.deepStrictEqual(
    ChannelEpoch.toAccount(ChannelEpoch.fromAccount(channelEpoch)),
    ok(channelEpoch),
  )
  const unfinalized = { ...channelEpoch, finalTotals: null }
  assert.deepStrictEqual(
    ChannelEpoch.toAccount(ChannelEpoch.fromAccount(unfinalized)),
    ok(unfinalized),
  )
  assert.deepStrictEqual(
    BidderPosition.toAccount(BidderPosition.fromAccount(bidderPosition)),
    ok(bidderPosition),
  )
})

test("both stored formats of a slope read as one number", () => {
  const defaults = EpochTerms.fromAccount({
    rampBonusSlopeMbps: 1_500_000,
    exitPenaltyExponent: 2,
  })
  const snapshot = EpochTerms.fromStored({
    rampBonusSlopeQ64f64: 3n * HALF,
    exitPenaltyExponent: 2,
  })
  assert.equal(DecimalNumber.format(defaults.rampBonusSlope), "1.5")
  assert.equal(DecimalNumber.format(snapshot.rampBonusSlope), "1.5")
})

test("a number that the stored format cannot hold is an error", () => {
  const tenth = { value: 1n, decimals: 1 }
  const tenMillionth = { value: 1n, decimals: 7 }
  const negative = { value: -1n, decimals: 0 }

  // 0.1 is 100,000 micro-basis points, but it is not a multiple of 2^-64.
  const terms = { rampBonusSlope: tenth, exitPenaltyExponent: 2 }
  assert.deepStrictEqual(
    EpochTerms.toAccount(terms),
    ok({ rampBonusSlopeMbps: 100_000, exitPenaltyExponent: 2 }),
  )
  assert.deepStrictEqual(EpochTerms.toStored(terms), err("Inexact"))
  assert.deepStrictEqual(
    EpochTerms.toAccount({ ...terms, rampBonusSlope: tenMillionth }),
    err("Inexact"),
  )

  // A failure anywhere inside a model is the failure of the whole model.
  const channelModel = Channel.fromAccount(channel)
  assert.deepStrictEqual(
    Channel.toAccount({
      ...channelModel,
      defaultEpochTerms: { ...terms, rampBonusSlope: negative },
    }),
    err("NegativeToUnsigned"),
  )

  const channelEpochModel = ChannelEpoch.fromAccount(channelEpoch)
  assert.deepStrictEqual(
    ChannelEpoch.toAccount({
      ...channelEpochModel,
      weightedSharesBalance: { value: 1n << 64n, decimals: 0 },
    }),
    err("Overflow"),
  )
  assert.deepStrictEqual(
    ChannelEpoch.toAccount({ ...channelEpochModel, terms }),
    err("Inexact"),
  )
  assert.deepStrictEqual(
    ChannelEpoch.toAccount({
      ...channelEpochModel,
      finalTotals: { rewardPot: { amount: 300n }, weightedShares: negative },
    }),
    err("NegativeToUnsigned"),
  )

  const position = BidderPosition.fromAccount(bidderPosition)
  assert.deepStrictEqual(
    BidderPosition.toAccount({
      ...position,
      lots: [...position.lots, { credits: { amount: 1n }, shares: tenth }],
    }),
    err("Inexact"),
  )
})
