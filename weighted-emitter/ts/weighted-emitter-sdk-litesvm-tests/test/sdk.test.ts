import assert from "node:assert/strict"
import { test } from "node:test"
import { DecimalNumber } from "@mayflower-sys/fixed-number-ts"
import { depositVoteInstruction } from "@mayflower-sys/weighted-emitter-sdk-ts"
import { readEmitter, readGauge, readVotePosition } from "./prelude/accounts.ts"
import { createFixture, instructions, startTime } from "./prelude/fixture.ts"
import { programId, setTime } from "./prelude/runtime.ts"

// These examples exercise the public SDK against the real program. Program
// mathematics, custody, and authorization cases belong to the Rust suites.
test("creates emitter, gauge and vote position with SDK-derived addresses", () => {
  const f = createFixture()
  const emitter = readEmitter(f.emitter)
  assert.partialDeepStrictEqual(emitter, {
    authority: { bytes: f.authority.publicKey.toBytes() },
    voteMint: { bytes: f.voteMint.toBytes() },
    emissionMint: { bytes: f.emissionMint.toBytes() },
    lastUpdatedAtUnixSeconds: { seconds: startTime },
    totalDeposited: { amount: 0n },
  })
  assert.equal(DecimalNumber.format(emitter.emissionRate), "12.5")
  assert.equal(DecimalNumber.format(emitter.annualDecayRate), "0")
  assert.partialDeepStrictEqual(readGauge(f.gauge), {
    emitter: { bytes: f.emitter.toBytes() },
    totalDeposited: { amount: 0n },
    lastUpdatedAtUnixSeconds: { seconds: startTime },
  })
  assert.partialDeepStrictEqual(readVotePosition(f.votePosition), {
    authority: { bytes: f.voter.publicKey.toBytes() },
    gauge: { bytes: f.gauge.toBytes() },
    voteEscrow: { bytes: f.voteEscrow.toBytes() },
    deposited: { amount: 0n },
  })
})

test("deposits and withdraws votes through standalone and bound SDK builders", () => {
  const f = createFixture()
  f.bank.send(
    [
      depositVoteInstruction(programId, {
        ...f.selectedPosition,
        voteSource: f.voteSource,
        amount: 80n,
      }),
    ],
    [f.voter],
  )
  assert.equal(readVotePosition(f.votePosition).deposited.amount, 80n)

  f.bank.send(
    [
      instructions.withdrawVote({
        ...f.selectedPosition,
        voteDestination: f.voteSource,
        amount: 30n,
      }),
    ],
    [f.voter],
  )
  assert.equal(readVotePosition(f.votePosition).deposited.amount, 50n)
  assert.equal(readGauge(f.gauge).totalDeposited.amount, 50n)
  assert.equal(readEmitter(f.emitter).totalDeposited.amount, 50n)
})

test("updates the decay rate using a plain decimal SDK model", () => {
  const f = createFixture()
  f.bank.send(
    [
      instructions.setEmitterDecayRate({
        authority: f.authority.publicKey,
        emitter: f.emitter,
        annualDecayRate: { value: 125n, decimals: 3 },
      }),
    ],
    [f.authority],
  )
  assert.equal(
    DecimalNumber.format(readEmitter(f.emitter).annualDecayRate),
    "0.125",
  )
})

test("settles emission and reads nonzero Q192.64 indexes through SDK models", () => {
  const f = createFixture()
  f.bank.send(
    [
      instructions.depositVote({
        ...f.selectedPosition,
        voteSource: f.voteSource,
        amount: 10n,
      }),
    ],
    [f.voter],
  )
  setTime(startTime + 1n)
  f.bank.send(
    [
      instructions.settleEmission({
        authority: f.authority.publicKey,
        emitter: f.emitter,
        gauge: f.gauge,
        emissionMint: f.emissionMint,
        emissionDestination: f.emissionDestination,
      }),
    ],
    [f.authority],
  )
  const emitter = readEmitter(f.emitter)
  const gauge = readGauge(f.gauge)
  assert.equal(emitter.lastUpdatedAtUnixSeconds.seconds, startTime + 1n)
  assert.equal(gauge.lastUpdatedAtUnixSeconds.seconds, startTime + 1n)
  assert(emitter.emissionIndex.value > 0n)
  assert.equal(emitter.emissionIndex.decimals, 64)
  assert.deepEqual(gauge.emissionIndexSeen, emitter.emissionIndex)
  assert(gauge.totalMinted.amount > 0n)
})
