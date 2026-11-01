//! The error table is the seam a host generates its own error type from, so
//! its shape matters as much as its contents.

use mayf_reward_clearing::{RewardClearingError, with_reward_clearing_errors};

macro_rules! collect_errors {
    ($($variant:ident = $message:literal,)*) => {
        const TABLE: &[(RewardClearingError, &str)] =
            &[$((RewardClearingError::$variant, $message),)*];
    };
}

with_reward_clearing_errors!(collect_errors);

#[test]
fn every_variant_has_a_dense_ordinal_matching_its_table_position() {
    for (position, (error, _)) in TABLE.iter().enumerate() {
        assert_eq!(
            error.ordinal(),
            position as u32,
            "{error:?} does not sit at its table position"
        );
    }
}

#[test]
fn every_variant_carries_its_own_message() {
    for (error, message) in TABLE {
        assert_eq!(&error.message(), message);
        assert!(!message.is_empty(), "{error:?} has no message");
    }

    let mut messages: Vec<&str> = TABLE.iter().map(|(_, message)| *message).collect();
    messages.sort_unstable();
    let total = messages.len();
    messages.dedup();
    assert_eq!(messages.len(), total, "two errors share one message");
}

#[test]
fn a_host_generated_enum_stays_aligned_with_the_table() {
    // Stands in for a host's framework-specific enum: same variants, same
    // order, numbered from its own base. Generating it cannot drift.
    macro_rules! host_errors {
        ($($variant:ident = $message:literal,)*) => {
            #[derive(Debug, PartialEq, Eq)]
            #[repr(u32)]
            enum HostError { $($variant,)* }

            impl From<RewardClearingError> for HostError {
                fn from(error: RewardClearingError) -> Self {
                    match error { $(RewardClearingError::$variant => Self::$variant,)* }
                }
            }
        };
    }
    with_reward_clearing_errors!(host_errors);

    // Position, not code: a host adds its own base to this, so alignment here
    // is what makes any base produce matching codes on both sides.
    for (error, _) in TABLE {
        assert_eq!(HostError::from(*error) as u32, error.ordinal());
    }
}
