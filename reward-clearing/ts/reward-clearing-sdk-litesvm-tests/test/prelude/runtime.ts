import assert from "node:assert/strict"
import { fileURLToPath } from "node:url"
import {
  Keypair,
  PublicKey,
  Transaction,
  type TransactionInstruction,
} from "@solana/web3.js"
import { FailedTransactionMetadata, LiteSVM } from "litesvm"

export const programId = new PublicKey(
  "RCgecPmsA1YfbwBZ9FkPmGUkKqnEGwvMrpnzUhUthxd",
)
const artifact = fileURLToPath(
  new URL(
    "../../../../sol-programs/reward-clearing/bin/mayf_reward_clearing_sol.so",
    import.meta.url,
  ),
)

// One VM for the sequential suite. Only wallets, SPL mints/accounts, and program
// accounts created by a test belong to that test. Never seed program state.
export const svm = new LiteSVM()
try {
  svm.addProgramFromFile(programId, artifact)
} catch (cause) {
  throw new Error(
    `Cannot load ${artifact}; run reward-clearing/sol-programs/reward-clearing/build.sh`,
    { cause },
  )
}

export const createUser = (): Keypair => {
  const user = Keypair.generate()
  const result = svm.airdrop(user.publicKey, 10_000_000_000n)
  assert(
    !(result instanceof FailedTransactionMetadata),
    result?.toString() ?? "Airdrop failed",
  )
  assert.equal(svm.getBalance(user.publicKey), 10_000_000_000n)
  return user
}

export const setTime = (seconds: bigint): void => {
  const clock = svm.getClock()
  clock.unixTimestamp = seconds
  svm.setClock(clock)
}

export const account = (address: PublicKey) => {
  const result = svm.getAccount(address)
  assert(result, `Missing account ${address.toBase58()}`)
  return result
}

export const assertRentExempt = (address: PublicKey): void => {
  const info = account(address)
  assert(
    BigInt(info.lamports)
      >= svm.minimumBalanceForRentExemption(BigInt(info.data.length)),
    `Account ${address.toBase58()} must be rent exempt`,
  )
}

/** A fresh fee payer keeps transaction fees apart from rent and token balances. */
export class Bank {
  readonly feePayer = createUser()

  private execute(instructions: TransactionInstruction[], signers: Keypair[]) {
    // Retries must reach the program, with signature and blockhash checks intact.
    svm.expireBlockhash()
    const tx = new Transaction({
      feePayer: this.feePayer.publicKey,
      recentBlockhash: svm.latestBlockhash(),
    }).add(...instructions)
    const unique = new Map(
      [this.feePayer, ...signers].map((key) => [key.publicKey.toBase58(), key]),
    )
    tx.sign(...unique.values())
    return svm.sendTransaction(tx)
  }

  send(instructions: TransactionInstruction[], signers: Keypair[]) {
    const result = this.execute(instructions, signers)
    if (result instanceof FailedTransactionMetadata) {
      assert.fail(`${result.toString()}\n${result.meta().logs().join("\n")}`)
    }
    return result
  }

  /** Assert an exact instruction error and rollback of every supplied account. */
  reject(
    instructions: TransactionInstruction[],
    signers: Keypair[],
    instructionIndex: number,
    errorCode: number,
  ): FailedTransactionMetadata {
    const addresses = new Map(
      instructions.flatMap((ix) =>
        ix.keys
          .filter((key) => !key.pubkey.equals(this.feePayer.publicKey))
          .map((key) => [key.pubkey.toBase58(), key.pubkey] as const),
      ),
    )
    const before = [...addresses.values()].map(
      (key) => [key, svm.getAccount(key)] as const,
    )
    const result = this.execute(instructions, signers)
    assert(result instanceof FailedTransactionMetadata, "Expected rejection")
    const error = result.err()
    assert(typeof error === "object" && "err" in error, result.toString())
    assert.equal(error.index, instructionIndex, result.toString())
    const detail = error.err()
    assert(typeof detail === "object" && "code" in detail, result.toString())
    assert.equal(detail.code, errorCode, result.toString())
    for (const [key, original] of before) {
      assert.deepEqual(svm.getAccount(key), original, key.toBase58())
    }
    return result
  }
}
