import assert from "node:assert/strict"
import { test } from "node:test"
import { DecimalNumber } from "@mayflower-sys/fixed-number-ts"
import { type EpochTerms } from "@mayflower-sys/reward-clearing-sdk-ts"
import { TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { SystemProgram, type Keypair, type PublicKey } from "@solana/web3.js"
import {
  readBidderPosition,
  readChannel,
  readChannelEpoch,
  readChannelGroup,
  readEpoch,
  readOrganization,
} from "./prelude/accounts.ts"
import {
  createFixture,
  defaultTerms,
  duration,
  initialTokens,
  instructions,
  pda,
  startTime,
} from "./prelude/fixture.ts"
import {
  account,
  assertRentExempt,
  createUser,
  setTime,
  svm,
} from "./prelude/runtime.ts"
import { readMint, readTokenAccount } from "./prelude/tokens.ts"

// The runner executes this file sequentially. Each case creates its own wallets,
// mints and organization; the shared VM and clock never imply shared test state.
test("creates accounts through the SDK and reads their relationships as models", () => {
  const f = createFixture()
  assert.deepEqual(readOrganization(f.organization), {
    authority: { bytes: f.authority.publicKey.toBytes() },
    nextChannelGroupIndex: 1,
  })
  assert.partialDeepStrictEqual(readChannelGroup(f.channelGroup), {
    organization: { bytes: f.organization.toBytes() },
    clearingCreditsMint: { bytes: f.creditsMint.address.toBytes() },
    channelGroupIndex: 0,
    currentEpochIndex: 0,
    masterStartTimeUnixSeconds: { seconds: startTime },
    epochDurationSeconds: duration,
  })
  assert.deepEqual(readEpoch(f.epoch), {
    channelGroup: { bytes: f.channelGroup.toBytes() },
    epochCounter: 0,
    startTimeUnixSeconds: { seconds: startTime },
    durationSeconds: duration,
  })
  const channel = readChannel(f.channel)
  assert.partialDeepStrictEqual(channel, {
    channelGroup: { bytes: f.channelGroup.toBytes() },
    rewardTokenMint: { bytes: f.rewardMint.address.toBytes() },
    rewardStagingTokenAccount: { bytes: f.staging.toBytes() },
    lastCreatedEpochCounter: 0,
    disabled: false,
    defaultEpochTerms: { exitPenaltyExponent: 1 },
  })
  assert.equal(
    DecimalNumber.format(channel.defaultEpochTerms.rampBonusSlope),
    "0",
  )
  const channelEpoch = readChannelEpoch(f.channelEpoch)
  assert.partialDeepStrictEqual(channelEpoch, {
    channel: { bytes: f.channel.toBytes() },
    epochCounter: 0,
    rewardPotEscrowTokenAccount: { bytes: f.rewardPot.toBytes() },
    clearingCreditsEscrowTokenAccount: { bytes: f.creditsEscrow.toBytes() },
    clearingCreditsBalance: { amount: 0n },
    rewardPotBalance: { amount: 0n },
    terms: { exitPenaltyExponent: 1 },
    finalTotals: null,
  })
  assert.equal(DecimalNumber.format(channelEpoch.weightedSharesBalance), "0")
  assert.equal(DecimalNumber.format(channelEpoch.terms.rampBonusSlope), "0")
  assert.partialDeepStrictEqual(readBidderPosition(f.position), {
    authority: { bytes: f.bidder.publicKey.toBytes() },
    channel: { bytes: f.channel.toBytes() },
    epochCounter: 0,
    claimed: false,
    lots: [],
  })

  for (const address of [
    f.organization,
    f.channelGroup,
    f.epoch,
    f.channel,
    f.channelEpoch,
    f.position,
  ]) {
    assertRentExempt(address)
  }
  const vaults: [PublicKey, PublicKey, PublicKey][] = [
    [f.groupCredits, f.creditsMint.address, f.channelGroup],
    [f.staging, f.rewardMint.address, f.channel],
    [f.rewardPot, f.rewardMint.address, f.channelEpoch],
    [f.creditsEscrow, f.creditsMint.address, f.channelEpoch],
  ]
  for (const [address, mint, owner] of vaults) {
    assert(account(address).owner.equals(TOKEN_PROGRAM_ID))
    assertRentExempt(address)
    const token = readTokenAccount(address)
    assert(token.mint.equals(mint))
    assert(token.owner.equals(owner))
    assert.equal(token.amount, 0n)
  }

  // Read the updated counter; a second group must use index 1, not a fixed 0.
  const { nextChannelGroupIndex } = readOrganization(f.organization)
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
  const second = pda.channelGroup(f.organization, nextChannelGroupIndex)
  assert.equal(readChannelGroup(second).channelGroupIndex, 1)
  assert.equal(readOrganization(f.organization).nextChannelGroupIndex, 2)
})

test("administers a channel and lets an unrelated payer open the next epoch", () => {
  const f = createFixture()
  const originalEpoch = account(f.channelEpoch)
  const terms: EpochTerms.EpochTerms = {
    rampBonusSlope: { value: 15n, decimals: 1 },
    exitPenaltyExponent: 2,
  }
  f.bank.send(
    [
      instructions.setChannelDefaultEpochTerms({
        authority: f.authority.publicKey,
        channelGroup: f.channelGroup,
        channel: f.channel,
        defaultEpochTerms: terms,
      }),
      instructions.setChannelDisabled({
        authority: f.authority.publicKey,
        channelGroup: f.channelGroup,
        channel: f.channel,
        disabled: true,
      }),
    ],
    [f.authority],
  )
  const updated = readChannel(f.channel)
  assert.equal(updated.disabled, true)
  assert.equal(
    DecimalNumber.format(updated.defaultEpochTerms.rampBonusSlope),
    "1.5",
  )
  assert.equal(updated.defaultEpochTerms.exitPenaltyExponent, 2)
  f.bank.send(
    [
      instructions.setChannelDisabled({
        authority: f.authority.publicKey,
        channelGroup: f.channelGroup,
        channel: f.channel,
        disabled: false,
      }),
    ],
    [f.authority],
  )

  setTime(startTime + BigInt(duration))
  const caller = createUser()
  f.bank.send(
    [
      instructions.createEpoch({
        payer: caller.publicKey,
        channelGroup: f.channelGroup,
        currentEpochIndex: readChannelGroup(f.channelGroup).currentEpochIndex,
      }),
    ],
    [caller],
  )
  const { currentEpochIndex } = readChannelGroup(f.channelGroup)
  assert.equal(currentEpochIndex, 1)
  assert.deepEqual(readEpoch(pda.epoch(f.channelGroup, currentEpochIndex)), {
    channelGroup: { bytes: f.channelGroup.toBytes() },
    epochCounter: 1,
    startTimeUnixSeconds: { seconds: startTime + BigInt(duration) },
    durationSeconds: duration,
  })
  f.bank.send(
    [
      instructions.createChannelEpoch({
        payer: caller.publicKey,
        channelGroup: f.channelGroup,
        currentEpochIndex,
        rewardTokenMint: f.rewardMint.address,
        clearingCreditsMint: f.creditsMint.address,
      }),
    ],
    [caller],
  )
  const next = readChannelEpoch(pda.channelEpoch(f.channel, currentEpochIndex))
  assert.equal(next.epochCounter, 1)
  assert.equal(DecimalNumber.format(next.terms.rampBonusSlope), "1.5")
  assert.equal(next.terms.exitPenaltyExponent, 2)
  assert.deepEqual(
    next.rewardPotEscrowTokenAccount.bytes,
    pda.channelEpochRewardPotEscrow(f.channel, currentEpochIndex).toBytes(),
  )
  assert.deepEqual(
    next.clearingCreditsEscrowTokenAccount.bytes,
    pda
      .channelEpochClearingCreditsEscrow(f.channel, currentEpochIndex)
      .toBytes(),
  )
  assert.equal(readChannel(f.channel).lastCreatedEpochCounter, 1)
  assert.equal(readChannel(f.channel).disabled, false)
  // Updating defaults and creating a new epoch cannot rewrite existing terms.
  assert.deepEqual(account(f.channelEpoch), originalEpoch)
})

test("deposits credits, funds position growth, and withdraws with a burn and refund", () => {
  const f = createFixture()
  const empty = account(f.position)
  const payerBefore = account(f.payer.publicKey).lamports
  f.bank.send(
    [
      instructions.depositCredits({
        ...f.selectedEpoch,
        authority: f.bidder.publicKey,
        payer: f.payer.publicKey,
        clearingCreditsSource: f.creditsSource,
        amount: 100n,
      }),
    ],
    [f.bidder, f.payer],
  )
  const deposited = readBidderPosition(f.position)
  assert.equal(deposited.lots.length, 1)
  assert.equal(deposited.lots[0]!.credits.amount, 100n)
  assert.equal(DecimalNumber.format(deposited.lots[0]!.shares), "100")
  assert.equal(
    readChannelEpoch(f.channelEpoch).clearingCreditsBalance.amount,
    100n,
  )
  assert.equal(readTokenAccount(f.creditsSource).amount, initialTokens - 100n)
  assert.equal(readTokenAccount(f.creditsEscrow).amount, 100n)
  assert.equal(readMint(f.creditsMint.address).supply, initialTokens)
  const grown = account(f.position)
  assert(grown.data.length > empty.data.length)
  assertRentExempt(f.position)
  assert.equal(
    payerBefore - account(f.payer.publicKey).lamports,
    grown.lamports - empty.lamports,
  )

  // One exact halfway example checks CPI wiring; curve permutations belong in Rust.
  setTime(startTime + BigInt(duration / 2))
  f.bank.send(
    [
      instructions.withdrawCredits({
        ...f.selectedEpoch,
        authority: f.bidder.publicKey,
        clearingCreditsMint: f.creditsMint.address,
        clearingCreditsDestination: f.creditsSource,
        amount: 100n,
      }),
    ],
    [f.bidder],
  )
  assert.equal(readTokenAccount(f.creditsSource).amount, initialTokens - 50n)
  assert.equal(readTokenAccount(f.creditsEscrow).amount, 0n)
  assert.equal(readMint(f.creditsMint.address).supply, initialTokens - 50n)
  const withdrawn = readChannelEpoch(f.channelEpoch)
  assert.equal(withdrawn.clearingCreditsBalance.amount, 0n)
  assert.equal(DecimalNumber.format(withdrawn.weightedSharesBalance), "0")
  // The reader must handle unused trailing bytes after the lot vector shrinks.
  assert.deepEqual(readBidderPosition(f.position).lots, [])
  assert.equal(account(f.position).data.length, grown.data.length)
  assert.equal(account(f.position).lamports, grown.lamports)
})

test("funds rewards, finalizes a historical epoch, and claims tokens and position rent", () => {
  const f = createFixture()
  f.bank.send(
    [
      instructions.depositCredits({
        ...f.selectedEpoch,
        authority: f.bidder.publicKey,
        payer: f.payer.publicKey,
        clearingCreditsSource: f.creditsSource,
        amount: 100n,
      }),
      instructions.addToRewardPot({
        ...f.selectedEpoch,
        transferAuthority: f.authority.publicKey,
        rewardTokenSource: f.rewardSource,
        amount: 250n,
      }),
    ],
    [f.bidder, f.payer, f.authority],
  )
  assert.equal(readTokenAccount(f.rewardSource).amount, initialTokens - 250n)
  assert.equal(readTokenAccount(f.rewardPot).amount, 250n)
  assert.equal(readChannelEpoch(f.channelEpoch).rewardPotBalance.amount, 250n)

  setTime(startTime + BigInt(duration))
  // Advance the group so the explicit historical epoch selection matters.
  const caller = createUser()
  f.bank.send(
    [
      instructions.createEpoch({
        payer: caller.publicKey,
        channelGroup: f.channelGroup,
        currentEpochIndex: readChannelGroup(f.channelGroup).currentEpochIndex,
      }),
      instructions.finalizeChannelEpoch({
        ...f.selectedEpoch,
        clearingCreditsMint: f.creditsMint.address,
      }),
    ],
    [caller],
  )
  const finalized = readChannelEpoch(f.channelEpoch)
  assert(finalized.finalTotals)
  assert.equal(finalized.finalTotals.rewardPot.amount, 250n)
  assert.equal(
    DecimalNumber.format(finalized.finalTotals.weightedShares),
    "100",
  )
  // The stored credit total is historical; finalization burns its physical custody.
  assert.equal(finalized.clearingCreditsBalance.amount, 100n)
  assert.equal(readTokenAccount(f.creditsEscrow).amount, 0n)
  assert.equal(readMint(f.creditsMint.address).supply, initialTokens - 100n)

  const rent = account(f.position).lamports
  const bidderBefore = account(f.bidder.publicKey).lamports
  const claim = instructions.claimRewards({
    authority: f.bidder.publicKey,
    channel: f.channel,
    epochCounter: f.epochCounter,
    rewardDestination: f.rewardDestination,
  })
  f.bank.send([claim], [f.bidder])
  assert.equal(readTokenAccount(f.rewardDestination).amount, 250n)
  assert.equal(readTokenAccount(f.rewardPot).amount, 0n)
  assert.equal(readMint(f.rewardMint.address).supply, initialTokens)
  // LiteSVM 0.2 retains a zeroed account until cleanup; newer versions return null.
  const closed = svm.getAccount(f.position)
  if (closed !== null) {
    assert.equal(closed.lamports, 0)
    assert.equal(closed.data.length, 0)
    assert(closed.owner.equals(SystemProgram.programId))
    assert.equal(closed.executable, false)
  }
  assert.equal(account(f.bidder.publicKey).lamports, bidderBefore + rent)
  const claimed = readChannelEpoch(f.channelEpoch)
  assert.equal(claimed.rewardPotBalance.amount, 0n)
  assert.equal(DecimalNumber.format(claimed.weightedSharesBalance), "0")
  assert.deepEqual(claimed.finalTotals, finalized.finalTotals)
  f.bank.reject([claim], [f.bidder], 0, 3012) // Anchor: AccountNotInitialized
})

test("administration requires the organization's authority and its signature", () => {
  const f = createFixture()
  const impostor = createUser()
  // A valid second organization rules out an accidental missing-account failure.
  f.bank.send(
    [
      instructions.createOrganization({
        authority: impostor.publicKey,
        payer: f.payer.publicKey,
      }),
    ],
    [impostor, f.payer],
  )
  for (const authority of [impostor.publicKey, f.authority.publicKey]) {
    const actions = [
      instructions.setChannelDisabled({
        authority,
        channelGroup: f.channelGroup,
        channel: f.channel,
        disabled: true,
      }),
      instructions.setChannelDefaultEpochTerms({
        authority,
        channelGroup: f.channelGroup,
        channel: f.channel,
        defaultEpochTerms: { ...defaultTerms, exitPenaltyExponent: 2 },
      }),
    ]
    for (const ix of actions) {
      if (authority.equals(impostor.publicKey)) {
        f.bank.reject([ix], [impostor], 0, 2001) // Anchor: ConstraintHasOne
      } else {
        // A hostile transaction can clear the builder's signer flag.
        ix.keys.find((key) => key.pubkey.equals(authority))!.isSigner = false
        f.bank.reject([ix], [], 0, 3010) // Anchor: AccountNotSigner
      }
    }
  }
})

test("deposits, withdrawals and claims require the recorded bidder's signature", () => {
  const f = createFixture()
  const impostor = createUser()
  // Keep the real position in each hostile message; changing builder parameters
  // alone would derive the impostor's nonexistent position instead.
  const deposit = () =>
    instructions.depositCredits({
      ...f.selectedEpoch,
      authority: f.bidder.publicKey,
      payer: f.payer.publicKey,
      clearingCreditsSource: f.creditsSource,
      amount: 1n,
    })
  const withdraw = () =>
    instructions.withdrawCredits({
      ...f.selectedEpoch,
      authority: f.bidder.publicKey,
      clearingCreditsMint: f.creditsMint.address,
      clearingCreditsDestination: f.creditsSource,
      amount: 1n,
    })
  const claim = () =>
    instructions.claimRewards({
      authority: f.bidder.publicKey,
      channel: f.channel,
      epochCounter: f.epochCounter,
      rewardDestination: f.rewardDestination,
    })
  const actions: [typeof deposit, Keypair[]][] = [
    [deposit, [f.payer]],
    [withdraw, []],
    [claim, []],
  ]
  for (const [build, rentSigners] of actions) {
    const forged = build()
    forged.keys.find((key) => key.pubkey.equals(f.bidder.publicKey))!.pubkey =
      impostor.publicKey
    f.bank.reject([forged], [impostor, ...rentSigners], 0, 2001)
    const unsigned = build()
    unsigned.keys.find((key) =>
      key.pubkey.equals(f.bidder.publicKey),
    )!.isSigner = false
    f.bank.reject([unsigned], rentSigners, 0, 3010)
  }
})

test("reward funding enforces the source token owner and mint", () => {
  const f = createFixture()
  // The bidder owns the credits source, but does not own the reward source.
  for (const [source, code] of [
    [f.rewardSource, 4], // SPL Token: OwnerMismatch
    [f.creditsSource, 3], // SPL Token: MintMismatch
  ] as const) {
    f.bank.reject(
      [
        instructions.addToRewardPot({
          ...f.selectedEpoch,
          transferAuthority: f.bidder.publicKey,
          rewardTokenSource: source,
          amount: 1n,
        }),
      ],
      [f.bidder],
      0,
      code,
    )
  }
})

test("a failed refund rolls back earlier writes, the credit burn and token supply", () => {
  const f = createFixture()
  f.bank.send(
    [
      instructions.depositCredits({
        ...f.selectedEpoch,
        authority: f.bidder.publicKey,
        payer: f.payer.publicKey,
        clearingCreditsSource: f.creditsSource,
        amount: 100n,
      }),
    ],
    [f.bidder, f.payer],
  )
  setTime(startTime + BigInt(duration / 2))
  const failure = f.bank.reject(
    [
      instructions.setChannelDisabled({
        authority: f.authority.publicKey,
        channelGroup: f.channelGroup,
        channel: f.channel,
        disabled: true,
      }),
      instructions.withdrawCredits({
        ...f.selectedEpoch,
        authority: f.bidder.publicKey,
        clearingCreditsMint: f.creditsMint.address,
        clearingCreditsDestination: f.rewardDestination,
        amount: 40n,
      }),
    ],
    [f.authority, f.bidder],
    1,
    3, // SPL Token: MintMismatch
  )
  // SPL burned credits before rejecting the refund. reject() checks all supplied
  // accounts, including the earlier channel write, mint supply and bidder rent.
  const tokenCalls = failure
    .meta()
    .logs()
    .filter((log) => log.startsWith("Program log: Instruction:"))
  assert.deepEqual(tokenCalls, [
    "Program log: Instruction: Burn",
    "Program log: Instruction: Transfer",
  ])
})
