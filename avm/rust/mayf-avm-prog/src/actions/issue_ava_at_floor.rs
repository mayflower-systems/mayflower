use mayf_avm_calculator::AvmCalculator;
use mayf_fixed_number::SignedFixed;

pub use super::issue_ava::{
    IssueAvaArgs as IssueAvaAtFloorArgs, IssueAvaError as IssueAvaAtFloorError,
    IssueAvaReceipt as IssueAvaAtFloorReceipt,
};

/// Issue AVA at the current floor for an authorized option exercise.
/// The new AVA extends the floor tail and preserves characteristic progress.
pub fn issue_ava_at_floor<N: SignedFixed>(
    args: IssueAvaAtFloorArgs<N>,
) -> Result<IssueAvaAtFloorReceipt<N>, IssueAvaAtFloorError> {
    let position_next = args.credited_position()?;
    let quote = AvmCalculator::new(args.market.ramp_scalar)
        .quote_issue_at_floor(&args.market.calculator_state, args.amount_ava)
        .map_err(IssueAvaAtFloorError::Calculator)?;
    args.settle(position_next, quote, args.market.fee_rates.exercise_fee)
}
