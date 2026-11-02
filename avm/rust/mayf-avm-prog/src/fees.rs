use mayf_micro_basis_points::MicroBasisPoints;

pub struct FeeChain {
    /// amount of fee paid to the market owner
    pub fee_to_market_owner: u64,
    /// amount fee paid to the protocol
    pub fee_to_protocol: u64,
}

pub fn split_fee(gross_fee_amount: u64, protocol_fee_rate: MicroBasisPoints) -> FeeChain {
    // first take protocol fee from the gross fee amount
    // round up to nearest integer (defensive in favor of the protocol)
    // then the remainder is the fee to the market owner

    todo!()
}
