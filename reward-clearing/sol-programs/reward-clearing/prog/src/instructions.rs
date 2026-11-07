pub(crate) mod channel_create;
pub(crate) mod channel_epoch_add_to_reward_pot;
pub(crate) mod channel_epoch_create;
pub(crate) mod channel_epoch_finalize;
pub(crate) mod channel_set_default_epoch_terms;
pub(crate) mod channel_set_disabled;
pub(crate) mod create_organization;
pub(crate) mod create_tenant;
pub(crate) mod epoch_create;

// `#[program]` resolves each instruction's generated client-account modules at
// the crate root, so these globs must reach the root through `lib.rs`.
pub(crate) use channel_create::*;
pub(crate) use channel_epoch_add_to_reward_pot::*;
pub(crate) use channel_epoch_create::*;
pub(crate) use channel_epoch_finalize::*;
pub(crate) use channel_set_default_epoch_terms::*;
pub(crate) use channel_set_disabled::*;
pub(crate) use create_organization::*;
pub(crate) use create_tenant::*;
pub(crate) use epoch_create::*;
