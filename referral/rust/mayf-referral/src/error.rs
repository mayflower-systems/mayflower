//! Semantic failures returned by referral state transitions.

/// A failure while validating or applying referral membership or scoped balances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferralError {
    /// The configured depth or override rate is outside the supported membership policy.
    InvalidConfiguration,
    /// The current scheduled join price exceeds the buyer's limit.
    JoinFeeExceeded,
    /// A persisted node path is malformed.
    InvalidNodePath,
    /// A persisted offer cannot have arisen from offer publication.
    InvalidOffer,
    /// A persisted lane contains invalid direct or subtree accounting.
    InvalidLaneAccounting,
    /// Two values belong to different referral networks.
    NetworkMismatch,
    /// A root does not refer to the supplied node.
    RootNodeMismatch,
    /// An offer does not refer to the supplied parent node.
    OfferParentMismatch,
    /// The parent already has an open active offer.
    ActiveOfferExists,
    /// The parent has no active offer.
    NoActiveOffer,
    /// The supplied offer is closed.
    OfferClosed,
    /// The supplied offer is not the parent's current version.
    OfferVersionMismatch,
    /// A replacement attempted to reuse its predecessor's identifier.
    OfferIdReused,
    /// A node is already at the maximum permitted depth.
    MaximumDepth,
    /// A new node identifier already occurs in its inherited path.
    DuplicatePathNode,
    /// A lane does not match the corresponding node path entry.
    LanePathMismatch,
    /// A lane belongs to a different accounting scope.
    LaneScopeMismatch,
    /// The signer is not the source node's recorded owner.
    LaneOwnerMismatch,
    /// A lane deposit must contain a positive amount.
    LaneDepositTooSmall,
    /// A lane withdrawal must contain a positive amount.
    LaneWithdrawalTooSmall,
    /// A lane withdrawal exceeded the source's direct balance.
    LaneWithdrawalExceedsDirectBalance,
    /// Checked integer arithmetic failed.
    ArithmeticFailure,
}

/// Result type used by this crate.
pub type Result<T> = core::result::Result<T, ReferralError>;
