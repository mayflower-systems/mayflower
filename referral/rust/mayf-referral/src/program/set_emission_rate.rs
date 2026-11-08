use mayf_fixed_number::FixedNumber;

use crate::{EmissionRate, Protocol, Seconds};

pub struct SetEmissionRateArgs<N> {
    pub protocol: Protocol<N>,
    pub emission_rate: EmissionRate,
    pub now: Seconds,
}

pub struct SetEmissionRateReceipt<N> {
    pub protocol_next: Protocol<N>,
}

/// Emit at a new rate from `now` on. Emission up to `now` keeps the old rate.
pub fn set_emission_rate<N>(args: SetEmissionRateArgs<N>) -> SetEmissionRateReceipt<N>
where
    N: FixedNumber,
{
    let protocol = args.protocol;
    let protocol_next = Protocol {
        emitter: protocol.emitter.with_rate(args.emission_rate, args.now),
        ..protocol
    };

    SetEmissionRateReceipt { protocol_next }
}
