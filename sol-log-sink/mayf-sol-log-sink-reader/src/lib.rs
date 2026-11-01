//! Read log-sink payloads from decoded Solana instruction traces.
//!
//! [`logged_by`] selects calls made directly by a given program. It uses stack
//! heights to identify the immediate caller, including nested CPIs.
//!
//! Supply a successful transaction's legacy message and complete instruction
//! trace. The reader does not fetch transactions or check their status.

use solana_address::Address;
use solana_message::{Message, inner_instruction::InnerInstructionsList};

/// The log sink program ID in binary and base58 form.
pub use mayf_sol_log_sink_id::{ID, ID_STR};

/// Stack height of a CPI made by a top-level instruction.
const FIRST_INNER_STACK_HEIGHT: u8 = 2;

/// Invalid instruction-trace structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceError {
    /// Trace group count differs from the top-level instruction count.
    InstructionGroupCount,
    /// An instruction's program index is outside the message's account keys.
    ProgramIndex,
    /// A stack height has no valid caller in the active invocation stack.
    StackHeight,
}

impl std::fmt::Display for TraceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InstructionGroupCount => {
                "inner-instruction group count does not match the message"
            }
            Self::ProgramIndex => "instruction program index is outside the message account keys",
            Self::StackHeight => "inner instruction has an impossible stack height",
        })
    }
}

impl std::error::Error for TraceError {}

/// Return borrowed payloads sent directly by `caller`, in execution order.
///
/// Accepts a legacy [`Message`] and a complete inner-instruction trace.
/// Provide one trace group per top-level instruction, in order, including
/// empty groups for instructions with no CPIs. The caller must check that
/// the transaction succeeded before using the result.
///
/// # Errors
///
/// Returns [`TraceError`] for an invalid group count, program index, or stack
/// transition. No partial result is returned.
pub fn logged_by<'a>(
    message: &Message,
    inner_instructions: &'a InnerInstructionsList,
    caller: &Address,
) -> Result<Vec<&'a [u8]>, TraceError> {
    if inner_instructions.len() != message.instructions.len() {
        return Err(TraceError::InstructionGroupCount);
    }
    let program = |index: u8| {
        message
            .account_keys
            .get(usize::from(index))
            .ok_or(TraceError::ProgramIndex)
    };
    let mut payloads = Vec::new();
    for (instruction, inner) in message.instructions.iter().zip(inner_instructions) {
        let mut stack = vec![program(instruction.program_id_index)?];
        for invoked in inner {
            let height = usize::from(invoked.stack_height);
            if height < usize::from(FIRST_INNER_STACK_HEIGHT) || height > stack.len() + 1 {
                return Err(TraceError::StackHeight);
            }
            stack.truncate(height - 1);
            let invoker = stack[height - 2];
            let invoked_program = program(invoked.instruction.program_id_index)?;
            if invoked_program == &ID && invoker == caller {
                payloads.push(invoked.instruction.data.as_slice());
            }
            stack.push(invoked_program);
        }
    }
    Ok(payloads)
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_message::{
        compiled_instruction::CompiledInstruction, inner_instruction::InnerInstruction,
    };

    const PAYER: Address = Address::new_from_array([9; 32]);
    const WATCHED: Address = Address::new_from_array([5; 32]);
    const OTHER: Address = Address::new_from_array([7; 32]);

    fn call(program: u8, data: &[u8], stack_height: u8) -> InnerInstruction {
        InnerInstruction {
            instruction: CompiledInstruction::new_from_raw_parts(program, data.to_vec(), vec![]),
            stack_height,
        }
    }

    // Account keys: payer 0, watched program 1, other program 2, sink 3.
    fn message(top_level_programs: &[u8]) -> Message {
        Message {
            account_keys: vec![PAYER, WATCHED, OTHER, ID],
            instructions: top_level_programs
                .iter()
                .map(|&program| CompiledInstruction::new_from_raw_parts(program, vec![], vec![]))
                .collect(),
            ..Message::default()
        }
    }

    #[test]
    fn a_payload_belongs_to_the_program_one_stack_level_up() {
        // The watched program logs from the top. Then another program calls
        // the watched program, which logs, and then logs itself.
        let inner = vec![
            vec![call(3, b"direct", 2)],
            vec![
                call(1, b"", 2),
                call(3, b"nested", 3),
                call(3, b"forged", 2),
            ],
        ];
        assert_eq!(
            logged_by(&message(&[1, 2]), &inner, &WATCHED).unwrap(),
            vec![b"direct".as_slice(), b"nested".as_slice()]
        );
        assert_eq!(
            logged_by(&message(&[1, 2]), &inner, &OTHER).unwrap(),
            vec![b"forged".as_slice()]
        );
    }

    #[test]
    fn a_deeper_call_finds_its_nearest_caller() {
        // Another program calls the watched program twice. The second call's
        // log sits after the first call's log, which is at the same height.
        let inner = vec![vec![
            call(1, b"", 2),
            call(3, b"first", 3),
            call(2, b"", 2),
            call(1, b"", 3),
            call(3, b"second", 4),
        ]];
        assert_eq!(
            logged_by(&message(&[2]), &inner, &WATCHED).unwrap(),
            vec![b"first".as_slice(), b"second".as_slice()]
        );
        assert!(
            logged_by(&message(&[2]), &inner, &OTHER)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn only_calls_to_the_sink_are_payloads() {
        let inner = vec![vec![call(2, b"not a log", 2)]];
        assert!(
            logged_by(&message(&[1]), &inner, &WATCHED)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn malformed_traces_return_no_partial_payloads() {
        let message = message(&[1]);
        for height in [0, 1, 3, u8::MAX] {
            assert_eq!(
                logged_by(&message, &vec![vec![call(3, b"invalid", height)]], &WATCHED),
                Err(TraceError::StackHeight),
            );
        }
        assert_eq!(
            logged_by(&message, &vec![], &WATCHED),
            Err(TraceError::InstructionGroupCount)
        );
        assert_eq!(
            logged_by(&message, &vec![vec![], vec![]], &WATCHED),
            Err(TraceError::InstructionGroupCount)
        );
        assert_eq!(
            logged_by(&message, &vec![vec![call(4, b"invalid", 2)]], &WATCHED),
            Err(TraceError::ProgramIndex)
        );
        let mut invalid_top = message;
        invalid_top.instructions[0].program_id_index = 4;
        assert_eq!(
            logged_by(&invalid_top, &vec![vec![]], &WATCHED),
            Err(TraceError::ProgramIndex)
        );
    }

    #[test]
    fn a_completed_branch_cannot_supply_a_missing_caller() {
        let inner = vec![vec![
            call(2, b"", 2),
            call(1, b"", 3),
            call(3, b"valid", 4),
            call(2, b"", 2),
            call(3, b"stale-parent", 4),
        ]];
        assert_eq!(
            logged_by(&message(&[2]), &inner, &WATCHED),
            Err(TraceError::StackHeight)
        );
    }
}
