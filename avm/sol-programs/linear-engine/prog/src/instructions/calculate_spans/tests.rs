use super::*;
use crate::wire::Q64x64Bounds;
use anchor_lang::solana_program::program::MAX_RETURN_DATA;
use mayf_fixed_number::MathError;

const ONE: u128 = 1 << 64;

fn encloses(bounds: Q64x64Bounds, numerator: u128, denominator: u128) {
    assert!(bounds.lower <= bounds.upper);
    assert!(bounds.lower * denominator <= numerator);
    assert!(bounds.upper * denominator >= numerator);
}

#[test]
fn mixed_spans_enclose_linear_values_on_and_between_grid_points() {
    // Eighth-unit coordinates have exact products and integrals on Q64.64.
    let coordinates = [(0, 0), (6, 5), (4, 0), (0, 8)];
    let requests: Vec<_> = coordinates
        .map(|(lo, width)| SpanRequest {
            lo_q64x64: lo * ONE / 8,
            width_q64x64: width * ONE / 8,
        })
        .into();
    let responses = evaluate_spans(3 * ONE / 2, &requests).unwrap();
    for ((lo, width), response) in coordinates.into_iter().zip(responses) {
        for (bounds, exact) in [
            (response.y_at_lo, 3 * lo * ONE / 16),
            (response.y_at_hi, 3 * (lo + width) * ONE / 16),
            (response.area, 3 * width * (2 * lo + width) * ONE / 256),
        ] {
            assert_eq!(
                bounds,
                Q64x64Bounds {
                    lower: exact,
                    upper: exact
                }
            );
        }
    }

    // One-quantum widths exercise directed rounding, including an integral
    // smaller than one representable quantum. Expectations are exact ratios
    // in raw units, computed independently of the fixed-point implementation.
    let requests = [
        SpanRequest {
            lo_q64x64: 0,
            width_q64x64: 1,
        },
        SpanRequest {
            lo_q64x64: ONE,
            width_q64x64: 1,
        },
        SpanRequest {
            lo_q64x64: 1,
            width_q64x64: 0,
        },
    ];
    let responses = evaluate_spans(3 * ONE / 2, &requests).unwrap();
    encloses(responses[0].y_at_hi, 3, 2);
    encloses(responses[0].area, 3, 4 * ONE);
    encloses(responses[1].y_at_hi, 3 * (ONE + 1), 2);
    encloses(responses[1].area, 6 * ONE + 3, 4 * ONE);
    encloses(responses[2].y_at_lo, 3, 2);
    assert_eq!(responses[2].y_at_lo, responses[2].y_at_hi);
    assert_eq!(responses[2].area, Q64x64Bounds { lower: 0, upper: 0 });
    for response in responses {
        assert!(response.y_at_lo.upper - response.y_at_lo.lower <= 1);
        assert!(response.y_at_hi.upper - response.y_at_hi.lower <= 1);
        assert!(response.area.upper - response.area.lower <= 3);
    }
}

fn fails_with(requests: &[SpanRequest], slope: u128, name: &str) {
    let error = evaluate_spans(slope, requests).unwrap_err();
    let anchor_lang::error::Error::AnchorError(error) = error else {
        panic!("unexpected program error: {error:?}");
    };
    assert_eq!(error.error_name, name);
}

#[test]
fn batch_capacity_matches_serialized_return_data() {
    let request = SpanRequest {
        lo_q64x64: ONE,
        width_q64x64: ONE,
    };
    let requests = vec![request; MAX_BATCH_SPANS];
    let responses = evaluate_spans(ONE, &requests).unwrap();
    let mut encoded = Vec::new();
    responses.serialize(&mut encoded).unwrap();
    assert!(encoded.len() <= MAX_RETURN_DATA);
    assert_eq!(
        Vec::<SpanResponse>::try_from_slice(&encoded).unwrap(),
        responses
    );

    let mut oversized = responses;
    oversized.push(oversized[0]);
    encoded.clear();
    oversized.serialize(&mut encoded).unwrap();
    assert!(encoded.len() > MAX_RETURN_DATA);
    fails_with(&vec![request; MAX_BATCH_SPANS + 1], ONE, "BatchTooLarge");
    encoded.clear();
    evaluate_spans(ONE, &[])
        .unwrap()
        .serialize(&mut encoded)
        .unwrap();
    assert_eq!(encoded, 0_u32.to_le_bytes());
}

#[test]
fn invalid_parameters_and_unrepresentable_results_fail() {
    let point = SpanRequest {
        lo_q64x64: ONE,
        width_q64x64: 0,
    };
    fails_with(&[point], 0, "InvalidSlope");
    fails_with(
        &[SpanRequest {
            lo_q64x64: u128::MAX,
            width_q64x64: 1,
        }],
        ONE,
        "ArithmeticFailure",
    );
    fails_with(
        &[SpanRequest {
            lo_q64x64: u128::MAX,
            width_q64x64: 0,
        }],
        2 * ONE,
        "ArithmeticFailure",
    );
    // Endpoints fit, but the integral does not.
    fails_with(
        &[SpanRequest {
            lo_q64x64: 0,
            width_q64x64: 1 << 97,
        }],
        ONE,
        "ArithmeticFailure",
    );

    let mut response = evaluate_spans(ONE, &[point]).unwrap()[0];
    response.area = Q64x64Bounds { lower: 1, upper: 0 };
    assert_eq!(
        mayf_avm_calculator::SpanResponse::<U64F64>::try_from(response),
        Err(MathError::Domain),
    );
}
