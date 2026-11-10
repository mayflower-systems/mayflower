use mayf_fixed_drivers::U192F64;
use mayf_fixed_number::NonNeg;
use mayf_weighted_emitter::{
    CreateEmitterArgs, CreateGaugeArgs, EmitToGaugeArgs, Emitter, Gauge, GaugeReceipt,
    SetEmitterDecayRateArgs, SetGaugeVotesArgs, create_emitter, create_gauge, emit_to_gauge,
    set_emitter_decay_rate, set_gauge_votes,
};

pub type Number = U192F64;
pub type TestEmitter = Emitter<u64, Number>;
pub type TestGauge = Gauge<u64, Number>;

pub fn amount(value: u64) -> NonNeg<Number> {
    NonNeg::new(Number::from_u64(value)).unwrap()
}

/// A host that persists complete successful receipts. Tests control time explicitly.
#[derive(Clone)]
pub struct Fixture {
    pub emitter: TestEmitter,
    pub gauges: Vec<TestGauge>,
}

impl Fixture {
    pub fn new(rate: u64, weights: &[u64]) -> Self {
        let emitter = create_emitter(CreateEmitterArgs {
            id: 100,
            emission_rate: amount(rate),
            annual_decay_rate: amount(0),
            at: 0,
        })
        .unwrap()
        .new_emitter;
        let mut fixture = Self {
            emitter,
            gauges: Vec::new(),
        };
        for &weight in weights {
            let gauge = fixture.add_gauge(0);
            assert_eq!(fixture.vote(gauge, weight, 0), 0);
        }
        fixture
    }

    pub fn add_gauge(&mut self, at: u64) -> usize {
        let index = self.gauges.len();
        let receipt = create_gauge(CreateGaugeArgs {
            emitter: self.emitter.clone(),
            id: index as u64,
            at,
        })
        .unwrap();
        self.emitter = receipt.emitter_next;
        self.gauges.push(receipt.new_gauge);
        index
    }

    pub fn emit(&mut self, gauge: usize, at: u64) -> u64 {
        let receipt = emit_to_gauge(EmitToGaugeArgs {
            emitter: self.emitter.clone(),
            gauge: self.gauges[gauge].clone(),
            at,
        })
        .unwrap();
        self.apply(gauge, receipt)
    }

    pub fn vote(&mut self, gauge: usize, votes: u64, at: u64) -> u64 {
        let receipt = set_gauge_votes(SetGaugeVotesArgs {
            emitter: self.emitter.clone(),
            gauge: self.gauges[gauge].clone(),
            votes,
            at,
        })
        .unwrap();
        self.apply(gauge, receipt)
    }

    pub fn decay(&mut self, annual_decay: u64, at: u64) {
        self.emitter = set_emitter_decay_rate(SetEmitterDecayRateArgs {
            emitter: self.emitter.clone(),
            annual_decay_rate: amount(annual_decay),
            at,
        })
        .unwrap()
        .emitter_next;
    }

    fn apply(&mut self, gauge: usize, receipt: GaugeReceipt<u64, Number>) -> u64 {
        self.emitter = receipt.emitter_next;
        self.gauges[gauge] = receipt.gauge_next;
        assert_eq!(
            self.emitter.total_emitted,
            self.gauges
                .iter()
                .map(|gauge| gauge.total_emitted)
                .sum::<u64>()
        );
        receipt.emitted
    }
}
