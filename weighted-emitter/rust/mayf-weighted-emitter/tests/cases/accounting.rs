use mayf_fixed_number::FixedNumber;
use mayf_weighted_emitter::SECONDS_PER_YEAR;

use crate::support::{Fixture, Number, amount};

#[test]
fn proportional_allocations_reconcile_to_the_global_budget() {
    let mut host = Fixture::new(120, &[4, 2]);
    assert_eq!(host.emit(0, 10), 800);
    assert_eq!(host.emit(1, 10), 400);
    assert_eq!(host.emitter.total_emitted, 1_200);
}

#[test]
fn small_weight_allocations_never_exceed_an_independent_integer_oracle() {
    for left in 1..=12_u64 {
        for right in 1..=12_u64 {
            for rate in 1..=12_u64 {
                let mut host = Fixture::new(rate, &[left, right]);
                let elapsed = 5;
                let actual = [host.emit(0, elapsed), host.emit(1, elapsed)];
                for (emitted, weight) in actual.into_iter().zip([left, right]) {
                    let exact_floor = rate * elapsed * weight / (left + right);
                    assert!(emitted <= exact_floor);
                    // A precise index can lose at most the boundary token unit here.
                    assert!(exact_floor - emitted <= 1);
                }
                assert!(host.emitter.total_emitted <= rate * elapsed);
            }
        }
    }
}

#[test]
fn reweighting_materializes_the_old_interval_before_replacing_votes() {
    let mut host = Fixture::new(120, &[2, 2]);
    assert_eq!(host.vote(0, 0, 1), 60);
    assert_eq!(host.emit(1, 2), 180);
    assert_eq!(host.emit(0, 2), 0);
    assert_eq!(host.emitter.total_gauge_weight, 2);
    assert_eq!(host.emitter.total_emitted, 240);
}

#[test]
fn changing_one_gauge_does_not_require_materializing_the_others() {
    let mut host = Fixture::new(120, &[2, 2]);
    assert_eq!(host.vote(0, 4, 2), 120);
    assert_eq!(host.emit(0, 5), 240);
    assert_eq!(host.emit(1, 5), 240);
    assert_eq!(host.emitter.total_emitted, 600);
}

#[test]
fn zero_weight_intervals_never_become_backpay() {
    let mut host = Fixture::new(100, &[0]);
    assert_eq!(host.vote(0, 1, 50), 0);
    assert_eq!(host.emit(0, 51), 100);
    assert_eq!(host.vote(0, 0, 52), 100);
    assert_eq!(host.vote(0, 1, 100), 0);
    assert_eq!(host.emit(0, 101), 100);
    assert_eq!(host.emitter.total_emitted, 300);
}

#[test]
fn empty_intervals_still_decay_the_rate() {
    let mut host = Fixture::new(16, &[0]);
    host.decay(SECONDS_PER_YEAR, 0);
    assert_eq!(host.vote(0, 1, 2), 0);
    assert_eq!(host.emitter.schedule.emission_rate, amount(4));
    assert_eq!(host.emit(0, 3), 2);
}

#[test]
fn a_new_gauge_starts_at_the_current_index() {
    let mut host = Fixture::new(100, &[1]);
    let later = host.add_gauge(5);
    assert_eq!(
        host.gauges[later].emission_index_seen,
        host.emitter.emission_index
    );
    assert_eq!(host.vote(later, 1, 5), 0);
    assert_eq!(host.emit(later, 6), 50);
    assert_eq!(host.emit(0, 6), 550);
}

#[test]
fn repeated_snapshot_cannot_materialize_the_same_allocation_twice() {
    let mut host = Fixture::new(120, &[1]);
    assert_eq!(host.emit(0, 5), 600);
    let emitter = host.emitter.clone();
    let gauge = host.gauges[0].clone();
    assert_eq!(host.emit(0, 5), 0);
    assert_eq!(host.emitter, emitter);
    assert_eq!(host.gauges[0], gauge);
}

#[test]
fn each_materialization_discards_its_fraction_instead_of_carrying_it() {
    let mut frequent = Fixture::new(1, &[1, 1]);
    let mut delayed = frequent.clone();
    for at in 1..=4 {
        assert_eq!(frequent.emit(0, at), 0);
    }
    assert_eq!(delayed.emit(0, 4), 2);
    assert_eq!(frequent.emit(1, 4), 2);
    assert_eq!(frequent.gauges[0].total_emitted, 0);
    assert_eq!(frequent.emitter.total_emitted, 2);
}

#[test]
fn decay_changes_are_prospective_and_never_raise_the_current_rate() {
    let mut host = Fixture::new(16, &[1]);
    host.decay(SECONDS_PER_YEAR, 3);
    assert_eq!(host.emitter.schedule.emission_rate, amount(16));
    // Three old constant-rate seconds, then two half-rate ticks: 48 + 8 + 4.
    assert_eq!(host.emit(0, 5), 60);
    assert_eq!(host.emitter.schedule.emission_rate, amount(4));
    host.decay(0, 5);
    assert_eq!(host.emit(0, 7), 8);
    assert_eq!(host.emitter.schedule.emission_rate, amount(4));
}

#[test]
fn zero_rate_stays_zero_across_long_gaps_and_vote_changes() {
    let mut host = Fixture::new(0, &[1]);
    host.decay(SECONDS_PER_YEAR, 0);
    assert_eq!(host.vote(0, 2, u64::MAX - 1), 0);
    assert_eq!(host.emit(0, u64::MAX), 0);
    assert_eq!(host.emitter.schedule.emission_rate, amount(0));
}

#[test]
fn one_gauge_can_materialize_the_entire_u64_range() {
    let mut host = Fixture::new(u64::MAX, &[1]);
    assert_eq!(host.emit(0, 1), u64::MAX);
    assert_eq!(host.emitter.total_emitted, u64::MAX);
    assert_eq!(host.gauges[0].total_emitted, u64::MAX);
}

#[test]
fn the_index_retains_fractions_far_below_a_token_base_unit() {
    let mut host = Fixture::new(1, &[1, 3]);
    assert_eq!(host.emit(0, 1), 0);
    let quarter = Number::ONE.checked_div_floor(Number::from_u64(4)).unwrap();
    assert_eq!(host.emitter.emission_index.get(), quarter);
    assert_eq!(host.gauges[0].emission_index_seen.get(), quarter);
}
