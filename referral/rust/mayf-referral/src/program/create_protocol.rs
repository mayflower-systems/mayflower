use crate::{NetworkTerms, Protocol};

pub struct CreateProtocolArgs {
    pub terms: NetworkTerms,
}

pub struct CreateProtocolReceipt {
    pub new_protocol: Protocol,
}

/// Start an empty network with validated, immutable tree terms.
pub fn create_protocol(args: CreateProtocolArgs) -> CreateProtocolReceipt {
    CreateProtocolReceipt {
        new_protocol: Protocol {
            terms: args.terms,
            lane_count: 0,
        },
    }
}
