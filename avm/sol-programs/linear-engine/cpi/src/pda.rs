//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]
use crate::ProgramAddress;

// pda derivations ---------------------------------------------------

/// PDA derivations scoped to one program deployment.
pub trait MayfAvmEngineLinearPda: ProgramAddress {}

impl<T: ProgramAddress + ?Sized> MayfAvmEngineLinearPda for T {}
