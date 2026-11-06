//! Pure auction transitions: numeric state in, replacement state and amounts out.
//!
//! Each action lives in its own module under `actions/`. An action takes a
//! state snapshot and explicit arguments and returns a receipt with the
//! computed amounts and replacement state.

mod add_reward;
mod advance_epoch;
mod claim;
mod deposit;
mod deposit_credits;
mod settle;
mod withdraw;

pub use add_reward::{AddRewardArgs, AddRewardReceipt, add_reward};
pub use advance_epoch::{AdvanceEpochArgs, AdvanceEpochReceipt, advance_epoch};
pub use claim::{ClaimArgs, ClaimReceipt, ClaimState, claim};
pub use deposit::{DepositArgs, DepositReceipt, DepositState, deposit, entry_weight};
pub use settle::{SettleArgs, SettleReceipt, SettleState, settle};
pub use withdraw::{WithdrawArgs, WithdrawReceipt, WithdrawState, withdraw, withdrawal_penalty};
