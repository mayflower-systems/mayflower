use mayf_fixed_number::IntegralRounding;

use crate::{Amount, Identifiable, Node, Result};

pub struct CollectCreditsArgs<I, N> {
    pub member: Node<I, N>,
}

pub struct CollectCreditsReceipt<I, N> {
    pub member_next: Node<I, N>,
    /// The credits the host mints to the member's owner
    pub credits_collected: Amount,
}

/// Mint only already-accrued net entitlement; preserve the individual fraction.
/// This does not settle sources. To collect through a snapshot, first settle
/// the member's own lanes with [`settle_node_in_lane`](super::settle_node_in_lane)
/// and incoming gifts with [`settle_family`](super::settle_family). A zero-asset
/// child needs only the latter. Collection never changes escrow principal.
pub fn collect_credits<I, N>(args: CollectCreditsArgs<I, N>) -> Result<CollectCreditsReceipt<I, N>>
where
    I: Identifiable,
    N: IntegralRounding,
{
    let (member_next, credits_collected) = args.member.collect()?;

    Ok(CollectCreditsReceipt {
        member_next,
        credits_collected,
    })
}
