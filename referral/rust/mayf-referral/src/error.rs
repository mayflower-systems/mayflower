use mayf_fixed_number::MathError;

/// Every input the kernel rejects
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The maximum depth times the override rate is not below 100%, so the deepest members would
    /// keep nothing
    OverrideRateTooLargeForDepth,
    /// The parent is at the network's maximum depth, so it cannot admit children
    ParentAtMaxDepth,
    /// The lineage is not the node's: its parent first, each next node the parent of the one
    /// before, up to a top-level member
    BrokenLineage,
    /// A lane node belongs to no node of the member and its lineage, or repeats one
    StrayLaneNode,
    /// The withdrawal is more than the member's balance in the lane
    WithdrawalExceedsBalance,
    /// The host's fixed-point number type cannot hold a value the kernel computes
    NumberOutOfRange,
    /// The host's fixed-point number type rounds in steps coarser than 2^-64. Rounding could
    /// then move a credit base unit or more each time anyone settles.
    GridTooCoarse,
}

/// [`create_protocol`](crate::program::create_protocol) accepts only number types that hold
/// every value the kernel computes, and those values never go negative and are never divided by
/// zero. So after that, a fixed-point failure means the host broke a duty, for example by passing
/// a clock that went back.
impl From<MathError> for Error {
    fn from(_: MathError) -> Self {
        Self::NumberOutOfRange
    }
}

pub type Result<T> = core::result::Result<T, Error>;
