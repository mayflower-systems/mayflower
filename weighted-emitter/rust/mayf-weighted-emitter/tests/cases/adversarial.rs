use mayf_fixed_drivers::U256Scaled;
use mayf_fixed_number::NonNeg;
use mayf_weighted_emitter::{
    CreateEmitterArgs, EmitToGaugeArgs, SetGaugeVotesArgs, WeightedEmitterError, create_emitter,
    emit_to_gauge, set_gauge_votes,
};

use crate::support::{Fixture, amount};

#[test]
fn a_gauge_cannot_consume_another_emitters_index() {
    let mut host = Fixture::new(10, &[1]);
    host.gauges[0].emitter_id += 1;
    assert_eq!(
        emit_to_gauge(EmitToGaugeArgs {
            emitter: host.emitter,
            gauge: host.gauges.remove(0),
            at: 10,
        })
        .unwrap_err(),
        WeightedEmitterError::GaugeEmitterMismatch
    );
}

#[test]
fn regressed_time_is_rejected_without_a_receipt() {
    let mut host = Fixture::new(10, &[1]);
    host.emit(0, 5);
    assert_eq!(
        emit_to_gauge(EmitToGaugeArgs {
            emitter: host.emitter,
            gauge: host.gauges.remove(0),
            at: 4,
        })
        .unwrap_err(),
        WeightedEmitterError::TimeWentBack
    );
}

#[test]
fn gauge_state_cannot_be_ahead_of_its_canonical_emitter() {
    for corrupt_index in [false, true] {
        let mut host = Fixture::new(10, &[1]);
        if corrupt_index {
            host.gauges[0].emission_index_seen = amount(1);
        } else {
            host.gauges[0].last_updated_at_unix_seconds = 1;
        }
        assert_eq!(
            emit_to_gauge(EmitToGaugeArgs {
                emitter: host.emitter,
                gauge: host.gauges.remove(0),
                at: 10,
            })
            .unwrap_err(),
            WeightedEmitterError::InvalidState
        );
    }
}

#[test]
fn a_gauges_recorded_votes_cannot_exceed_the_emitter_total() {
    let mut host = Fixture::new(10, &[1]);
    host.gauges[0].votes = 2;
    assert_eq!(
        set_gauge_votes(SetGaugeVotesArgs {
            emitter: host.emitter,
            gauge: host.gauges.remove(0),
            votes: 0,
            at: 1,
        })
        .unwrap_err(),
        WeightedEmitterError::InvalidState
    );
}

#[test]
fn aggregate_vote_overflow_rejects_the_complete_transition() {
    let host = Fixture::new(0, &[u64::MAX, 0]);
    assert_eq!(
        set_gauge_votes(SetGaugeVotesArgs {
            emitter: host.emitter.clone(),
            gauge: host.gauges[1].clone(),
            votes: 1,
            at: 1,
        })
        .unwrap_err(),
        WeightedEmitterError::ArithmeticOverflow
    );
    assert_eq!(host.emitter.last_updated_at_unix_seconds, 0);
    assert_eq!(host.emitter.total_gauge_weight, u64::MAX);
    assert_eq!(host.gauges[1].votes, 0);
}

#[test]
fn cumulative_materialization_overflow_never_wraps_or_partially_applies() {
    let mut host = Fixture::new(u64::MAX, &[1]);
    host.emit(0, 1);
    let emitter = host.emitter.clone();
    let gauge = host.gauges[0].clone();
    assert_eq!(
        emit_to_gauge(EmitToGaugeArgs {
            emitter: host.emitter.clone(),
            gauge: host.gauges[0].clone(),
            at: 2,
        })
        .unwrap_err(),
        WeightedEmitterError::ArithmeticOverflow
    );
    assert_eq!(host.emitter, emitter);
    assert_eq!(host.gauges[0], gauge);
}

#[test]
fn one_unmaterialized_payout_must_fit_a_u64() {
    let host = Fixture::new(u64::MAX, &[1]);
    assert_eq!(
        emit_to_gauge(EmitToGaugeArgs {
            emitter: host.emitter,
            gauge: host.gauges.into_iter().next().unwrap(),
            at: 2,
        })
        .unwrap_err(),
        WeightedEmitterError::ArithmeticOverflow
    );
}

#[test]
fn emitter_total_overflow_is_checked_even_when_each_gauge_total_fits() {
    let mut host = Fixture::new(u64::MAX, &[1, 1]);
    assert_eq!(host.emit(0, 2), u64::MAX);
    assert_eq!(
        emit_to_gauge(EmitToGaugeArgs {
            emitter: host.emitter,
            gauge: host.gauges.remove(1),
            at: 2,
        })
        .unwrap_err(),
        WeightedEmitterError::ArithmeticOverflow
    );
}

#[test]
fn coarse_accounting_grids_are_rejected_at_initialization() {
    type Coarse = U256Scaled<1_000_000>;
    let zero = NonNeg::new(Coarse::ZERO).unwrap();
    assert_eq!(
        create_emitter(CreateEmitterArgs {
            id: 1_u64,
            emission_rate: zero,
            annual_decay_rate: zero,
            at: 0,
        })
        .unwrap_err(),
        WeightedEmitterError::InsufficientPrecision
    );
}
