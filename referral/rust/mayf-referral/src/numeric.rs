//! Concrete numeric policy for the Mayflower referral program.
//!
//! Token balances and the scope's total deposits fit `u64`. Clock ticks also
//! fit `u64`, so cumulative point-time needs up to 128 integral bits. The
//! program keeps 64 fractional bits and chooses the existing 256-bit driver
//! for that accounting. Both fixed-point formats have quantum `2^-64`, so
//! compact rates widen exactly without another rounding step.

/// Compact unsigned Q64.64 for override and sharing rates.
///
/// Rates must additionally satisfy their domain bounds. Network overrides are
/// rounded up from the configured rational and validated against the depth.
pub type ReferralRate = mayf_fixed_drivers::U64F64;

/// Unsigned Q192.64 for points, point-time, and cumulative sharing indexes.
///
/// Its 32-byte storage holds `u64::MAX * u64::MAX` whole units with 64
/// fractional bits. [`ReferralRate`] widens into this format exactly via
/// `ReferralPoints::from(rate)`. The choice is part of this program's numeric
/// policy; callers do not supply a `FixedNumber` implementation.
pub type ReferralPoints = mayf_fixed_drivers::U192F64;

/// The program's concrete configuration for the generic referral algorithms.
pub type ReferralTreeConfig = referral_tree::Config<ReferralPoints>;

/// The program's concrete numeric tree state, separate from membership.
///
/// The composing host supplies the fields and associates this state with a
/// member and scope. Every fixed-point field uses [`ReferralPoints`], including
/// the sharing fraction, which can be widened exactly from [`ReferralRate`].
pub type ReferralTreeNode = referral_tree::Node<ReferralPoints>;
