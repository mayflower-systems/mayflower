use crate::{
    Amount, FinalizedEpoch, Position, Result, RewardClearingError, Seconds,
    amount::from_amount,
    math::{arithmetic, positive},
    validation::validate_claim,
};
use mayf_fixed_number::{FixedNumber, IntegralRounding, NonNeg};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimState<N: FixedNumber> {
    pub epoch: FinalizedEpoch<N>,
    pub position: Position<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimArgs {
    pub now: Seconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimReceipt<N: FixedNumber> {
    pub shares_claimed: NonNeg<N>,
    /// Whole reward earned by this position. The host chooses how to credit
    /// or transfer the amount and enforces replay protection in its state.
    pub reward_earned: Amount,
    pub epoch: FinalizedEpoch<N>,
    pub position: Position<N>,
}

/// Claim once at the immutable settlement ratio, flooring the entitlement to
/// whole units. Unpaid units stay in the auction, never in a fractional user
/// balance. All eligible shares are retired even when the payout is zero.
pub fn claim<N: IntegralRounding>(
    state: ClaimState<N>,
    args: ClaimArgs,
) -> Result<ClaimReceipt<N>> {
    let ClaimState { epoch, position } = state;
    validate_claim(&epoch, &position, args.now)?;
    let reward_earned = calculate_claim_reward(&epoch, position.shares)?;
    let unclaimed_shares = arithmetic(epoch.unclaimed_shares.checked_sub(position.shares))?;
    let unclaimed_reward = epoch
        .unclaimed_reward
        .checked_sub(reward_earned)
        .ok_or(RewardClearingError::ArithmeticFailure)?;
    Ok(ClaimReceipt {
        shares_claimed: position.shares,
        reward_earned,
        epoch: FinalizedEpoch {
            unclaimed_shares,
            unclaimed_reward,
            ..epoch
        },
        position: Position {
            claimed: true,
            ..position
        },
    })
}

fn calculate_claim_reward<N: IntegralRounding>(
    epoch: &FinalizedEpoch<N>,
    shares: NonNeg<N>,
) -> Result<Amount> {
    if shares == NonNeg::zero() {
        return Ok(0);
    }
    let divisor = positive(epoch.settlement_shares)?;
    let pro_rata = arithmetic(
        from_amount::<N>(epoch.settlement_reward)?.checked_mul_div_floor(shares, divisor),
    )?;
    arithmetic(pro_rata.get().checked_floor_to_u64())
}
