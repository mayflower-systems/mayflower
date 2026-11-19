import { Buffer } from "node:buffer"
import {
  ACCOUNT_SIZE,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
  createInitializeAccount3Instruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  unpackAccount,
  unpackMint,
} from "@solana/spl-token"
import { Keypair, SystemProgram, type PublicKey } from "@solana/web3.js"
import { Bank, account, svm } from "./runtime.ts"

/** Real SPL initialization; return the authority so examples can mint later. */
export const createMint = (bank: Bank, payer: Keypair) => {
  const mint = Keypair.generate()
  const mintAuthority = Keypair.generate()
  bank.send(
    [
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: mint.publicKey,
        lamports: Number(svm.minimumBalanceForRentExemption(BigInt(MINT_SIZE))),
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeMint2Instruction(
        mint.publicKey,
        0,
        mintAuthority.publicKey,
        null,
      ),
    ],
    [payer, mint],
  )
  return { address: mint.publicKey, mintAuthority }
}

export type TestMint = ReturnType<typeof createMint>

/** Explicit token accounts keep examples independent of ATA conventions. */
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
  mint: TestMint,
  destination: PublicKey,
  amount: bigint,
): void => {
  bank.send(
    [
      createMintToInstruction(
        mint.address,
        destination,
        mint.mintAuthority.publicKey,
        amount,
      ),
    ],
    [mint.mintAuthority],
  )
}

export const readTokenAccount = (address: PublicKey) => {
  const info = account(address)
  return unpackAccount(address, { ...info, data: Buffer.from(info.data) })
}

export const readMint = (address: PublicKey) => {
  const info = account(address)
  return unpackMint(address, { ...info, data: Buffer.from(info.data) })
}
