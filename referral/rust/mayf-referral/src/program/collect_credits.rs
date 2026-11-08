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

/// Collect the whole credits a member keeps, from every lane. The fraction of a base unit stays
/// for later. Credits a lane has not settled yet stay in that lane until it settles, for example
/// by [`settle_node_in_lane`](super::settle_node_in_lane).
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
