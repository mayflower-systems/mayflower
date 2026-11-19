import assert from "node:assert/strict"
import { Buffer } from "node:buffer"
import { readFileSync } from "node:fs"
import { test } from "node:test"
import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js"
import * as sdk from "../src/index.ts"
import {
  addresses,
  authority,
  channelGroupIndex,
  clearingCreditsDestination,
  clearingCreditsMint,
  clearingCreditsSource,
  epochCounter,
  payer,
  programId,
  rewardDestination,
  rewardTokenMint,
  rewardTokenSource,
  transferAuthority,
} from "./addresses.ts"

interface IdlInstruction {
  name: string
  discriminator: number[]
  accounts: {
    name: string
    address?: string
    writable?: boolean
    signer?: boolean
  }[]
}

const idl = JSON.parse(
  readFileSync(
    new URL(
      "../../../sol-programs/reward-clearing/idl/mayf_reward_clearing_sol.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as { address: string; instructions: IdlInstruction[] }

// Read access flags and account order from the program's IDL, independently of
// the generated builders. Expected PDAs come from the Rust address vectors.
const assertInstruction = (
  name: string,
  instruction: TransactionInstruction,
  accounts: Record<string, PublicKey>,
  payload = Buffer.alloc(0),
) => {
  const definition = idl.instructions.find(
    (instruction) => instruction.name === name,
  )
  assert.ok(definition, name)
  assert.ok(instruction instanceof TransactionInstruction)
  assert.equal(instruction.programId.toBase58(), idl.address)
  assert.deepEqual(
    instruction.keys.map(({ pubkey, isSigner, isWritable }) => ({
      address: pubkey.toBase58(),
      isSigner,
      isWritable,
    })),
    definition.accounts.map((account) => {
      const address = account.address ?? accounts[account.name]?.toBase58()
      assert.ok(address, `${name}.${account.name}`)
      return {
        address,
        isSigner: account.signer ?? false,
        isWritable: account.writable ?? false,
      }
    }),
  )
  assert.deepEqual(
    instruction.data,
    Buffer.concat([Buffer.from(definition.discriminator), payload]),
  )
}

const u32 = (value: number) => {
  const bytes = Buffer.alloc(4)
  bytes.writeUInt32LE(value)
  return bytes
}
const u64 = (value: bigint) => {
  const bytes = Buffer.alloc(8)
  bytes.writeBigUInt64LE(value)
  return bytes
}

const defaultEpochTerms = {
  rampBonusSlope: { value: 15n, decimals: 1 },
  exitPenaltyExponent: 2,
}
const termsPayload = Buffer.concat([u32(1_500_000), Buffer.from([2])])
const amount = 0xfedcba9876543210n
const selectedEpoch = {
  channelGroup: addresses.channelGroup,
  channel: addresses.channel,
  epochCounter,
}
const epochAccounts = {
  channel_group: addresses.channelGroup,
  channel: addresses.channel,
  epoch: addresses.epoch,
  channel_epoch: addresses.channelEpoch,
}
const adminAccounts = {
  authority,
  organization: addresses.organization,
  channel_group: addresses.channelGroup,
  channel: addresses.channel,
}

test("create organization derives its PDA and supports a separate payer", () => {
  for (const sponsor of [authority, payer]) {
    assertInstruction(
      "create_organization",
      sdk.createOrganizationInstruction(programId, {
        authority,
        payer: sponsor,
      }),
      { authority, payer: sponsor, organization: addresses.organization },
    )
  }
})

test("create channel group always opens epoch zero", () => {
  const masterStartTimeUnixSeconds = 1_800_000_000n
  const epochDurationSeconds = 3_600
  assertInstruction(
    "create_channel_group",
    sdk.createChannelGroupInstruction(programId, {
      authority,
      payer: authority,
      nextChannelGroupIndex: channelGroupIndex,
      clearingCreditsMint,
      masterStartTimeUnixSeconds,
      epochDurationSeconds,
    }),
    {
      authority,
      payer: authority,
      organization: addresses.organization,
      channel_group: addresses.channelGroup,
      initial_epoch: addresses.initialEpoch,
      clearing_credits_mint: clearingCreditsMint,
      clearing_credits_escrow: addresses.clearingTokenEscrow,
    },
    Buffer.concat([u64(masterStartTimeUnixSeconds), u32(epochDurationSeconds)]),
  )
})

test("create channel derives its staging vault and current epoch escrows", () => {
  assertInstruction(
    "channel_create",
    sdk.createChannelInstruction(programId, {
      authority,
      payer: authority,
      channelGroup: addresses.channelGroup,
      currentEpochIndex: epochCounter,
      rewardTokenMint,
      clearingCreditsMint,
      defaultEpochTerms,
    }),
    {
      ...adminAccounts,
      payer: authority,
      current_epoch: addresses.epoch,
      reward_token_mint: rewardTokenMint,
      clearing_credits_mint: clearingCreditsMint,
      reward_staging_token_account: addresses.rewardStagingTokenAccount,
      initial_channel_epoch: addresses.channelEpoch,
      reward_pot_escrow_account: addresses.rewardPotEscrow,
      clearing_credits_escrow_account: addresses.clearingCreditsEscrow,
    },
    termsPayload,
  )
})

test("set default terms converts the SDK decimal without rounding", () => {
  assertInstruction(
    "channel_set_default_epoch_terms",
    sdk.setChannelDefaultEpochTermsInstruction(programId, {
      authority,
      channelGroup: addresses.channelGroup,
      channel: addresses.channel,
      defaultEpochTerms,
    }),
    adminAccounts,
    termsPayload,
  )
})

test("set disabled encodes both boolean values", () => {
  for (const disabled of [true, false]) {
    assertInstruction(
      "channel_set_disabled",
      sdk.setChannelDisabledInstruction(programId, {
        authority,
        channelGroup: addresses.channelGroup,
        channel: addresses.channel,
        disabled,
      }),
      adminAccounts,
      Buffer.from([Number(disabled)]),
    )
  }
})

test("create epoch selects the current and next counters", () => {
  assertInstruction(
    "epoch_create",
    sdk.createEpochInstruction(programId, {
      payer,
      channelGroup: addresses.channelGroup,
      currentEpochIndex: epochCounter,
    }),
    {
      payer,
      channel_group: addresses.channelGroup,
      current_epoch: addresses.epoch,
      new_epoch: addresses.nextEpoch,
    },
  )
  assert.throws(
    () =>
      sdk.createEpochInstruction(programId, {
        payer,
        channelGroup: addresses.channelGroup,
        currentEpochIndex: 0xffff_ffff,
      }),
    RangeError,
  )
})

test("create channel epoch selects the current counter and both vaults", () => {
  assertInstruction(
    "channel_epoch_create",
    sdk.createChannelEpochInstruction(programId, {
      payer,
      channelGroup: addresses.channelGroup,
      currentEpochIndex: epochCounter,
      rewardTokenMint,
      clearingCreditsMint,
    }),
    {
      payer,
      channel_group: addresses.channelGroup,
      channel: addresses.channel,
      current_epoch: addresses.epoch,
      new_channel_epoch: addresses.channelEpoch,
      reward_token_mint: rewardTokenMint,
      clearing_credits_mint: clearingCreditsMint,
      reward_pot_escrow_account: addresses.rewardPotEscrow,
      clearing_credits_escrow_account: addresses.clearingCreditsEscrow,
    },
  )
})

test("finalize selects the requested completed epoch without a signer", () => {
  assertInstruction(
    "channel_epoch_finalize",
    sdk.finalizeChannelEpochInstruction(programId, {
      ...selectedEpoch,
      clearingCreditsMint,
    }),
    {
      channel: addresses.channel,
      completed_epoch: addresses.epoch,
      completed_channel_epoch: addresses.channelEpoch,
      clearing_credits_mint: clearingCreditsMint,
      clearing_credits_escrow_token_account: addresses.clearingCreditsEscrow,
    },
  )
})

test("add reward pot preserves the transfer authority, source, and u64 amount", () => {
  assertInstruction(
    "channel_epoch_add_to_reward_pot",
    sdk.addToRewardPotInstruction(programId, {
      ...selectedEpoch,
      transferAuthority,
      rewardTokenSource,
      amount,
    }),
    {
      ...epochAccounts,
      transfer_authority: transferAuthority,
      reward_pot_escrow_token_account: addresses.rewardPotEscrow,
      reward_token_src: rewardTokenSource,
    },
    u64(amount),
  )
})

test("create bidder position derives the authority's position", () => {
  assertInstruction(
    "bidder_position_create",
    sdk.createBidderPositionInstruction(programId, {
      ...selectedEpoch,
      authority,
      payer: authority,
    }),
    {
      ...epochAccounts,
      authority,
      payer: authority,
      bidder_position: addresses.bidderPosition,
    },
  )
})

test("deposit selects the clearing vault and supports sponsored position growth", () => {
  assertInstruction(
    "bidder_position_deposit",
    sdk.depositCreditsInstruction(programId, {
      ...selectedEpoch,
      authority,
      payer,
      clearingCreditsSource,
      amount,
    }),
    {
      ...epochAccounts,
      authority,
      payer,
      bidder_position: addresses.bidderPosition,
      clearing_credits_escrow_token_account: addresses.clearingCreditsEscrow,
      clearing_credits_source: clearingCreditsSource,
    },
    u64(amount),
  )
})

test("withdraw selects the clearing mint and explicit destination", () => {
  assertInstruction(
    "bidder_position_withdraw",
    sdk.withdrawCreditsInstruction(programId, {
      ...selectedEpoch,
      authority,
      clearingCreditsMint,
      clearingCreditsDestination,
      amount,
    }),
    {
      ...epochAccounts,
      authority,
      bidder_position: addresses.bidderPosition,
      clearing_credits_mint: clearingCreditsMint,
      clearing_credits_escrow_token_account: addresses.clearingCreditsEscrow,
      clearing_credits_destination: clearingCreditsDestination,
    },
    u64(amount),
  )
})

test("claim uses the selected epoch and retains writable authority for rent return", () => {
  assertInstruction(
    "bidder_position_claim",
    sdk.claimRewardsInstruction(programId, {
      authority,
      channel: addresses.channel,
      epochCounter,
      rewardDestination,
    }),
    {
      authority,
      channel_epoch: addresses.channelEpoch,
      bidder_position: addresses.bidderPosition,
      reward_pot_escrow_token_account: addresses.rewardPotEscrow,
      reward_destination: rewardDestination,
    },
  )
})

test("unrepresentable default terms fail before an instruction can be returned", () => {
  for (const rampBonusSlope of [
    { value: 1n, decimals: 7 },
    { value: -1n, decimals: 0 },
    { value: 1n << 32n, decimals: 0 },
  ]) {
    const terms = { rampBonusSlope, exitPenaltyExponent: 2 }
    assert.throws(
      () =>
        sdk.createChannelInstruction(programId, {
          authority,
          payer: authority,
          channelGroup: addresses.channelGroup,
          currentEpochIndex: epochCounter,
          rewardTokenMint,
          clearingCreditsMint,
          defaultEpochTerms: terms,
        }),
      RangeError,
    )
    assert.throws(
      () =>
        sdk.setChannelDefaultEpochTermsInstruction(programId, {
          authority,
          channelGroup: addresses.channelGroup,
          channel: addresses.channel,
          defaultEpochTerms: terms,
        }),
      RangeError,
    )
  }
})

test("builders compose directly into a web3 transaction", () => {
  const transaction = new Transaction({
    feePayer: payer,
    recentBlockhash: PublicKey.default.toBase58(),
  }).add(
    sdk.createBidderPositionInstruction(programId, {
      ...selectedEpoch,
      authority,
      payer,
    }),
    sdk.depositCreditsInstruction(programId, {
      ...selectedEpoch,
      authority,
      payer,
      clearingCreditsSource,
      amount: 100n,
    }),
  )
  const message = transaction.compileMessage()
  assert.equal(message.instructions.length, 2)
  assert.equal(message.header.numRequiredSignatures, 2)
  assert.deepEqual(message.accountKeys.slice(0, 2), [payer, authority])
})

test("factories bind independent program addresses, including detached methods", () => {
  const otherProgramId = new PublicKey(new Uint8Array(32).fill(10))
  const factories = [
    [programId, sdk.createRewardClearingInstructions(programId)],
    [otherProgramId, sdk.createRewardClearingInstructions(otherProgramId)],
  ] as const
  const params = {
    ...selectedEpoch,
    authority,
    payer,
    clearingCreditsSource,
    amount,
  }
  const deposits = factories.map(([address, builders]) => {
    const { depositCredits } = builders
    const instruction = depositCredits(params)
    assert.deepEqual(
      instruction,
      sdk.depositCreditsInstruction(address, params),
    )
    assert.equal(instruction.programId.toBase58(), address.toBase58())
    assert.deepEqual(
      builders.createOrganization({ authority, payer }),
      sdk.createOrganizationInstruction(address, { authority, payer }),
    )
    return instruction
  })
  assert.notDeepEqual(deposits[0]!.keys, deposits[1]!.keys)
})

test("amounts cannot wrap into a different u64 transaction value", () => {
  const build = [
    (amount: bigint) =>
      sdk.addToRewardPotInstruction(programId, {
        ...selectedEpoch,
        transferAuthority,
        rewardTokenSource,
        amount,
      }),
    (amount: bigint) =>
      sdk.depositCreditsInstruction(programId, {
        ...selectedEpoch,
        authority,
        payer,
        clearingCreditsSource,
        amount,
      }),
    (amount: bigint) =>
      sdk.withdrawCreditsInstruction(programId, {
        ...selectedEpoch,
        authority,
        clearingCreditsMint,
        clearingCreditsDestination,
        amount,
      }),
  ]
  for (const instruction of build) {
    assert.doesNotThrow(() => instruction(0n))
    assert.doesNotThrow(() => instruction(0xffff_ffff_ffff_ffffn))
    for (const amount of [-1n, 1n << 64n]) {
      assert.throws(() => instruction(amount), RangeError)
    }
  }
})

test("schedule integers cannot be truncated or wrapped by the encoder", () => {
  const params = {
    authority,
    payer: authority,
    nextChannelGroupIndex: channelGroupIndex,
    clearingCreditsMint,
    masterStartTimeUnixSeconds: 1_800_000_000n,
    epochDurationSeconds: 3_600,
  }
  for (const masterStartTimeUnixSeconds of [-1n, 1n << 64n]) {
    assert.throws(
      () =>
        sdk.createChannelGroupInstruction(programId, {
          ...params,
          masterStartTimeUnixSeconds,
        }),
      RangeError,
    )
  }
  for (const epochDurationSeconds of [-1, 0.5, 0x1_0000_0000, NaN, Infinity]) {
    assert.throws(
      () =>
        sdk.createChannelGroupInstruction(programId, {
          ...params,
          epochDurationSeconds,
        }),
      RangeError,
    )
  }
})

test("an exit exponent cannot wrap into a different u8 value", () => {
  for (const exitPenaltyExponent of [-1, 0.5, 256, NaN, Infinity]) {
    const terms = { ...defaultEpochTerms, exitPenaltyExponent }
    assert.throws(
      () =>
        sdk.createChannelInstruction(programId, {
          authority,
          payer: authority,
          channelGroup: addresses.channelGroup,
          currentEpochIndex: epochCounter,
          rewardTokenMint,
          clearingCreditsMint,
          defaultEpochTerms: terms,
        }),
      RangeError,
    )
    assert.throws(
      () =>
        sdk.setChannelDefaultEpochTermsInstruction(programId, {
          authority,
          channelGroup: addresses.channelGroup,
          channel: addresses.channel,
          defaultEpochTerms: terms,
        }),
      RangeError,
    )
  }
})
