use mayf_fixed_number::FixedNumber;

use crate::{Offer, Protocol};

pub struct SetTopLevelOfferArgs<N> {
    pub protocol: Protocol<N>,
    pub offer: Offer,
}

pub struct SetTopLevelOfferReceipt<N> {
    pub protocol_next: Protocol<N>,
}

/// Set the price of a top-level seat. The next top-level member pays the new current fee.
pub fn set_top_level_offer<N>(args: SetTopLevelOfferArgs<N>) -> SetTopLevelOfferReceipt<N>
where
    N: FixedNumber,
{
    let protocol_next = Protocol {
        top_level_offer: args.offer,
        ..args.protocol
    };

    SetTopLevelOfferReceipt { protocol_next }
}
