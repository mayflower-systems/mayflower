//! Pure auction transitions: numeric state in, replacement state and amounts out.
//!
//! Each action lives in its own module under `actions/`. An action takes a
//! state snapshot and explicit arguments and returns a receipt with the
//! computed amounts and replacement state.

mod add_reward;
mod claim;
mod deposit;
mod settle;
mod start_epoch;
mod withdraw;

pub use add_reward::{AddRewardArgs, AddRewardReceipt, AddRewardState, add_reward};
pub use claim::{ClaimArgs, ClaimReceipt, ClaimState, claim};
pub use deposit::{DepositArgs, DepositReceipt, DepositState, deposit, entry_weight};
pub use settle::{SettleArgs, SettleReceipt, SettleState, settle};
pub use start_epoch::{StartEpochArgs, StartEpochReceipt, start_epoch};
pub use withdraw::{WithdrawArgs, WithdrawReceipt, WithdrawState, withdraw, withdrawal_penalty};
