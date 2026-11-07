use core::fmt::Debug;
use core::num::NonZeroU32;
use std::cell::Cell;

use mayf_avm_calculator::{
    Accounting, AvmCalculator, CharacteristicEngine, CharacteristicEngineResult, FloorRaisePolicy,
    LocalCharacteristic, MarketState, Segmentation, SpanRequest, SpanResponse,
    solver::{FloorRaiseSearch, SolverError, solve_floor_at_ramp_end, solve_floor_increase},
};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::{I64F64, I64Scaled};
use mayf_fixed_number::{
    AboveOne, Bounds, DirectedRounding, FixedNumber, NonNeg, Pos, SignedFixed,
};

fn n<N: FixedNumber>(numerator: i128, denominator: u128) -> NonNeg<N> {
    NonNeg::new(
        N::checked_from_ratio_round(
            numerator,
            denominator.try_into().unwrap(),
            DirectedRounding::Floor,
        )
        .unwrap(),
    )
    .unwrap()
}

fn positive<N: FixedNumber>(value: NonNeg<N>) -> Pos<N> {
    Pos::try_from(value).unwrap()
}

fn search<N: SignedFixed>() -> FloorRaiseSearch<N> {
    FloorRaiseSearch {
        max_iterations: NonZeroU32::new(12_000).unwrap(),
        max_floor_join_gap: n(0, 1),
        max_area_decrease: n(0, 1),
    }
}

struct Engine<N: FixedNumber> {
    local: LocalCharacteristic<LinearCharacteristic<N>>,
    calls: Cell<u32>,
}

impl<N: FixedNumber> Engine<N> {
    fn new(slope: NonNeg<N>) -> Self {
        Self {
            local: LocalCharacteristic::new(LinearCharacteristic::new(positive(slope))),
            calls: Cell::new(0),
        }
    }
}

impl<N: FixedNumber> CharacteristicEngine<N> for Engine<N> {
    fn spans(
        &self,
        requests: &[SpanRequest<N>],
        responses: &mut [SpanResponse<N>],
    ) -> CharacteristicEngineResult<()> {
        self.calls.set(self.calls.get() + 1);
        self.local.spans(requests, responses)
    }
}

fn seed_next(seed: &mut u64) -> i128 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    i128::from(*seed % 8 + 1)
}

fn check_seeded_linear_cases<N: SignedFixed + Debug>() {
    let mut seed = 0x27d4_eb2f_1656_67c5;
    for index in 0..24 {
        let slope = seed_next(&mut seed);
        let size = seed_next(&mut seed);
        let initial_floor = n(seed_next(&mut seed), 4);
        // Independent exact solutions of k*A^2 = (k-1)*E^2 for T=P.
        let (scalar, end, start, increase) = if index % 2 == 0 {
            (n(25, 16), 5 * size, 3 * size, n(15 * slope * size, 8))
        } else {
            (n(9, 8), 3 * size, size, n(3 * slope * size, 4))
        };
        let progress = n(end + seed_next(&mut seed), 1);
        let state = MarketState::new(
            Segmentation::launch(initial_floor),
            Accounting::new(progress, progress),
        )
        .unwrap();
        let calculator = AvmCalculator::new(AboveOne::new(scalar.get()).unwrap());
        let engine = Engine::new(n(slope, 1));
        let at_end = solve_floor_at_ramp_end(&calculator, &state, n(end, 1), search(), &engine)
            .unwrap_or_else(|error| panic!("case {index}, fixed end: {error:?}"));
        assert!(engine.calls.replace(0) <= search::<N>().max_iterations.get());
        let at_floor = solve_floor_increase(
            &calculator,
            &state,
            positive(increase),
            progress,
            search(),
            &engine,
        )
        .unwrap_or_else(|error| panic!("case {index}, fixed floor: {error:?}"));
        assert!(engine.calls.replace(0) <= search::<N>().max_iterations.get());
        for solution in [at_end, at_floor] {
            assert_eq!(solution.proposal.floor_increase_by, positive(increase));
            assert_eq!(solution.proposal.ramp_end, n(end, 1));
            assert_eq!(solution.proposal.ramp_width, n(end - start, 1));
            assert_eq!(solution.quote.backing_change, Bounds::exact(N::ZERO));
            assert_eq!(
                calculator
                    .verify_floor_raise(
                        &state,
                        solution.proposal,
                        FloorRaisePolicy::exact(),
                        &engine,
                    )
                    .unwrap(),
                solution.quote,
            );
        }
        engine.calls.set(0);
    }
}

#[test]
fn seeded_solvers_match_independent_linear_solutions_on_binary_and_decimal_grids() {
    check_seeded_linear_cases::<I64F64>();
    check_seeded_linear_cases::<I64Scaled<100_000>>();
}

#[test]
fn approximate_solution_obeys_the_same_one_sided_policy_as_the_program() {
    type N = I64Scaled<10_000>;
    let engine = Engine::<N>::new(n(1, 1));
    let calculator = AvmCalculator::new(AboveOne::new(n::<N>(2, 1).get()).unwrap());
    let state = MarketState::new(
        Segmentation::launch(n(1, 1)),
        Accounting::new(n(2, 1), n(2, 1)),
    )
    .unwrap();
    let limits = FloorRaiseSearch {
        max_floor_join_gap: n(1, 100),
        max_area_decrease: n(1, 100),
        ..search()
    };
    let solution = solve_floor_at_ramp_end(&calculator, &state, n(1, 1), limits, &engine).unwrap();
    let floor = solution.quote.next_state.segmentation().floor();
    // The ideal raised floor is sqrt(2); it is not on this decimal grid.
    assert!(floor >= n(140, 100) && floor <= n(142, 100));
    assert!(solution.quote.backing_change.upper() <= N::ZERO);
    assert!(solution.quote.backing_change.lower() >= n::<N>(1, 100).get().checked_neg().unwrap());
    assert_eq!(
        calculator
            .verify_floor_raise(
                &state,
                solution.proposal,
                FloorRaisePolicy::new(
                    limits.max_floor_join_gap,
                    limits.max_area_decrease.get().checked_neg().unwrap(),
                ),
                &engine,
            )
            .unwrap(),
        solution.quote,
    );
    assert_eq!(
        solve_floor_at_ramp_end(&calculator, &state, n(1, 1), search(), &engine),
        Err(SolverError::PrecisionLimit)
    );
}

#[test]
fn search_respects_progress_feasibility_and_the_engine_call_budget() {
    type N = I64F64;
    let engine = Engine::<N>::new(n(8, 1));
    let calculator = AvmCalculator::new(AboveOne::new(n::<N>(25, 16).get()).unwrap());
    let state = MarketState::new(
        Segmentation::launch(n(1, 1)),
        Accounting::new(n(6, 1), n(6, 1)),
    )
    .unwrap();
    assert_eq!(
        solve_floor_at_ramp_end(&calculator, &state, n(7, 1), search(), &engine),
        Err(SolverError::InvalidRange)
    );
    assert_eq!(engine.calls.get(), 0);
    assert_eq!(
        solve_floor_increase(
            &calculator,
            &state,
            positive(n(100, 1)),
            n(6, 1),
            search(),
            &engine,
        ),
        Err(SolverError::NoSolution)
    );
    engine.calls.set(0);
    assert_eq!(
        solve_floor_at_ramp_end(
            &calculator,
            &state,
            n(5, 1),
            FloorRaiseSearch {
                max_iterations: NonZeroU32::new(1).unwrap(),
                ..search()
            },
            &engine,
        ),
        Err(SolverError::IterationLimit)
    );
    assert_eq!(engine.calls.get(), 1);

    // With no physical floor tail, raising the floor to the existing ramp
    // start changes no backing. The root is the lower search endpoint.
    let engine = Engine::<N>::new(n(1, 1));
    let calculator = AvmCalculator::new(AboveOne::new(n::<N>(2, 1).get()).unwrap());
    let state = MarketState::new(
        Segmentation::new(n(2, 1), n(4, 1), n(1, 1), n::<N>(1, 1).get()).unwrap(),
        Accounting::new(n(4, 1), n(1, 1)),
    )
    .unwrap();
    let solution =
        solve_floor_at_ramp_end(&calculator, &state, n(4, 1), search(), &engine).unwrap();
    assert_eq!(solution.proposal.floor_increase_by, positive(n(1, 1)));
    assert_eq!(solution.proposal.ramp_width, n(1, 1));
    assert_eq!(solution.quote.backing_change, Bounds::exact(N::ZERO));
    let within_existing_gap = solve_floor_increase(
        &calculator,
        &state,
        positive(n(1, 2)),
        n(4, 1),
        FloorRaiseSearch {
            max_floor_join_gap: n(1, 2),
            ..search()
        },
        &engine,
    )
    .unwrap();
    assert_eq!(within_existing_gap.proposal.ramp_width, n(1, 1));
    assert_eq!(
        within_existing_gap.quote.backing_change,
        Bounds::exact(N::ZERO)
    );
}

#[test]
fn an_existing_ramp_and_independent_floor_tail_change_the_solved_floor() {
    type N = I64F64;
    let engine = Engine::<N>::new(n(8, 1));
    let calculator = AvmCalculator::new(AboveOne::new(n::<N>(25, 16).get()).unwrap());
    let limits = FloorRaiseSearch {
        max_floor_join_gap: n(1, 1_000_000_000),
        max_area_decrease: n(1, 1_000_000_000),
        ..search()
    };
    let mut floors = Vec::new();
    for circulation in [10, 12, 14] {
        let state = MarketState::new(
            Segmentation::new(n(16, 1), n(5, 1), n(2, 1), n::<N>(1, 1).get()).unwrap(),
            Accounting::new(n(12, 1), n(circulation, 1)),
        )
        .unwrap();
        let solution =
            solve_floor_at_ramp_end(&calculator, &state, n(10, 1), limits, &engine).unwrap();
        floors.push(solution.quote.next_state.segmentation().floor());
        assert!(solution.quote.backing_change.upper() <= N::ZERO);
        assert!(
            solution.quote.backing_change.lower()
                >= limits.max_area_decrease.get().checked_neg().unwrap()
        );
        let whole_change = calculator
            .backing(&solution.quote.next_state, &engine)
            .unwrap()
            .into_raw()
            .checked_sub(calculator.backing(&state, &engine).unwrap().into_raw())
            .unwrap();
        assert!(whole_change.lower() <= solution.quote.backing_change.upper());
        assert!(whole_change.upper() >= solution.quote.backing_change.lower());
    }
    assert!(floors[0] > floors[1] && floors[1] > floors[2]);
    assert!(floors[1] <= n(31, 1));
    assert!(n::<N>(31, 1).checked_sub(floors[1]).unwrap() < limits.max_area_decrease);
}
