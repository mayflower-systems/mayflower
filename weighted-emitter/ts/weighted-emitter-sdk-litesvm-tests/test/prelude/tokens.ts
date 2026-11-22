import {
  ACCOUNT_SIZE,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
  createInitializeAccount3Instruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
} from "@solana/spl-token"
import { Keypair, SystemProgram, type PublicKey } from "@solana/web3.js"
import { Bank, svm } from "./runtime.ts"

/** Explicit mint authority also allows an emitter's SDK-derived PDA. */
export const createMint = (
  bank: Bank,
  payer: Keypair,
  mintAuthority: PublicKey,
): PublicKey => {
  const mint = Keypair.generate()
  bank.send(
    [
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: mint.publicKey,
        lamports: Number(svm.minimumBalanceForRentExemption(BigInt(MINT_SIZE))),
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeMint2Instruction(mint.publicKey, 0, mintAuthority, null),
    ],
    [payer, mint],
  )
  return mint.publicKey
}

/** Create ordinary SPL accounts so examples do not depend on ATA conventions. */
export const createTokenAccount = (
  bank: Bank,
  payer: Keypair,
  mint: PublicKey,
  owner: PublicKey,
): PublicKey => {
  const token = Keypair.generate()
  bank.send(
    [
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: token.publicKey,
        lamports: Number(
          svm.minimumBalanceForRentExemption(BigInt(ACCOUNT_SIZE)),
        ),
        space: ACCOUNT_SIZE,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeAccount3Instruction(token.publicKey, mint, owner),
    ],
    [payer, token],
  )
  return token.publicKey
}

export const mintTokens = (
  bank: Bank,
  mint: PublicKey,
  mintAuthority: Keypair,
  destination: PublicKey,
  amount: bigint,
): void => {
  bank.send(
    [
      createMintToInstruction(
        mint,
        destination,
        mintAuthority.publicKey,
        amount,
      ),
    ],
    [mintAuthority],
  )
}
