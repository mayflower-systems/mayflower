# Reward clearing

Reward clearing distributes a funded pot through a timed auction. Bidders
commit clearing credits for shares of the pot. Entry time sets the weight of a
deposit, and withdrawal time sets the fraction of principal forfeited. At
settlement, the remaining credits are burned and the surviving shares
determine each payout.

Start with the auction design and runnable example in
[mayf-reward-clearing](rust/mayf-reward-clearing/README.md). To build the
on-chain program or call its instructions, see the
[Solana program guide](sol-programs/reward-clearing/README.md).
