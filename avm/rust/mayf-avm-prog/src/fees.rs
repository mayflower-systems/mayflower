use mayf_micro_basis_points::MicroBasisPoints;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeeChain {
    /// amount of fee paid to the market owner
    pub fee_to_market_owner: u64,
    /// amount fee paid to the protocol
    pub fee_to_protocol: u64,
}

/// Split a collected fee, rounding the protocol's share up to a reserve unit.
pub fn split_fee(gross_fee_amount: u64, protocol_fee_rate: MicroBasisPoints) -> FeeChain {
    let fee_to_protocol = protocol_fee_rate.apply_ceil(gross_fee_amount);

    FeeChain {
        fee_to_protocol,
        fee_to_market_owner: gross_fee_amount - fee_to_protocol,
    }
}
