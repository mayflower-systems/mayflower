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
  "6tdyEnBkiNYwbF5FmiF4Dq8piqq16CA642JEkMS88RgL",
)
const artifact = fileURLToPath(
  new URL(
    "../../../../sol-programs/weighted-emitter/bin/mayf_weighted_emitter_sol.so",
    import.meta.url,
  ),
)

// One VM for the sequential suite. Each test creates fresh wallets, mints,
// and program accounts through real instructions, without injecting state.
export const svm = new LiteSVM()
try {
  svm.addProgramFromFile(programId, artifact)
} catch (cause) {
  throw new Error(
    `Cannot load ${artifact}; run weighted-emitter/sol-programs/weighted-emitter/build.sh`,
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

/** Sign and submit web3.js instructions, keeping the fee payer separate. */
export class Bank {
  readonly feePayer = createUser()

  send(instructions: TransactionInstruction[], signers: Keypair[]) {
    svm.expireBlockhash()
    const tx = new Transaction({
      feePayer: this.feePayer.publicKey,
      recentBlockhash: svm.latestBlockhash(),
    }).add(...instructions)
    const unique = new Map(
      [this.feePayer, ...signers].map((key) => [key.publicKey.toBase58(), key]),
    )
    tx.sign(...unique.values())
    const result = svm.sendTransaction(tx)
    if (result instanceof FailedTransactionMetadata) {
      assert.fail(`${result.toString()}\n${result.meta().logs().join("\n")}`)
    }
    return result
  }
}
