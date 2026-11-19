import { PublicKey } from "@solana/web3.js"

export const programId = new PublicKey(
  "RCgecPmsA1YfbwBZ9FkPmGUkKqnEGwvMrpnzUhUthxd",
)
const key = (byte: number) => new PublicKey(new Uint8Array(32).fill(byte))

export const authority = key(1)
export const rewardTokenMint = key(2)
export const clearingCreditsMint = key(3)
export const payer = key(4)
export const transferAuthority = key(5)
export const rewardTokenSource = key(6)
export const clearingCreditsSource = key(7)
export const clearingCreditsDestination = key(8)
export const rewardDestination = key(9)

// Distinct bytes in each index detect the wrong byte order or integer width.
export const channelGroupIndex = 0x01020304
export const epochCounter = 0x05060708

// Produced independently with Rust's solana_address::Address::find_program_address
// and the seeds in the program's Accounts constraints, for the keys above.
export const addresses = {
  organization: new PublicKey("2ARhrLBJcYgRaHSS2v2zLCoXA7Gb7BpqLXxKHEHej64g"),
  channelGroup: new PublicKey("HRMb6pj7jQd72RSQAUaBWPXHrgRbt9KAbdjjPC3KYraj"),
  clearingTokenEscrow: new PublicKey(
    "89jRPhvqcH8RkHGn8NChSnGMqnmGyVWG21WDVfLpQzQQ",
  ),
  channel: new PublicKey("8Nz4sq9MEx7tMPR1mou6fnYeT3kRs314XDesfvbdVq37"),
  rewardStagingTokenAccount: new PublicKey(
    "4unnTg83mmEQZ9zhm21K6Y6Mz1bbid5b5RUTQEtmzaqV",
  ),
  initialEpoch: new PublicKey("8Npy6uPuuoT1mgQkP6nagxeC6WwvAQxg3AmB1jCGaCDg"),
  epoch: new PublicKey("5t4nTTVgKudKVr6Zqrf4aantqd9dTeKF21Xp9ttStTF8"),
  nextEpoch: new PublicKey("464CaH1FrydMP76iR3sAWwsH2FaGgoJLYZf7zdJro3Lu"),
  channelEpoch: new PublicKey("FBnAWg11iu71T46GqVdx1yE7YdwHUBi1oA4mi6hobaqC"),
  rewardPotEscrow: new PublicKey(
    "FykYxdQ8EzCzDJ3He47NKHBGBhoTMcWaCpSWJqJxhenf",
  ),
  clearingCreditsEscrow: new PublicKey(
    "4xQAQxwyHeDnGzYXpjhC8BM4X9NTFGrtdT8c1mhVo3Y3",
  ),
  bidderPosition: new PublicKey("74FpWtiCPxtSYMBwo46mBxJ1FsHYoVgLwVaRaeFKBec3"),
}
