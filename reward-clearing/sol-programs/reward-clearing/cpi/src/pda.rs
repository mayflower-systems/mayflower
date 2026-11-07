//! PDA addresses are supplied by callers because their seeds depend on account state.
//! Preserved by the generator's `--manual-pdas` mode.

use crate::ProgramAddress;

pub trait MayfRewardClearingPda: ProgramAddress {}

impl<T: ProgramAddress + ?Sized> MayfRewardClearingPda for T {}
