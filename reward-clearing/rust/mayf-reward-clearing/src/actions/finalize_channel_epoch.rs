use mayf_fixed_number::FixedNumber;

use crate::Result;
use crate::RewardClearingError;
use crate::Seconds;
use crate::models::Identifiable;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::channel_epoch_final::ChannelEpochFinal;
use crate::models::epoch::Epoch;

pub struct FinalizeChannelEpochArgs<I: Identifiable, N: FixedNumber> {
    pub channel_epoch: ChannelEpoch<I, N>,
    pub epoch: Epoch<I>,
    pub now: Seconds,
}

pub struct FinalizeChannelEpochReceipt<I: Identifiable, N: FixedNumber> {
    pub channel_epoch_next: ChannelEpoch<I, N>,
    pub channel_epoch_final: ChannelEpochFinal<I, N>,
}

/// Finalize a channel epoch once its epoch has ended.
/// The surviving weighted shares become claimable against the reward pot.
/// The host burns the remaining clearing credits.
///
/// The host must finalize each channel epoch only once.
/// A second final snapshot would reset the unclaimed shares and allow double claims.
pub fn finalize_channel_epoch<N, I>(
    args: FinalizeChannelEpochArgs<I, N>,
) -> Result<FinalizeChannelEpochReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    if !args.epoch.is_finished(args.now)? {
        return Err(RewardClearingError::EpochNotEnded);
    }

    args.channel_epoch.validate_epoch(&args.epoch)?;

    let channel_epoch_final = ChannelEpochFinal {
        channel_epoch_id: args.channel_epoch.id(),
        unclaimed_weighted_shares: args.channel_epoch.weighted_shares_balance,
    };

    Ok(FinalizeChannelEpochReceipt {
        channel_epoch_next: args.channel_epoch,
        channel_epoch_final,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RewardClearingError as E;
    use crate::test_support::*;

    fn finalize(
        channel_epoch: ChannelEpoch<Id, Number>,
        now: Seconds,
    ) -> Result<FinalizeChannelEpochReceipt<Id, Number>> {
        finalize_channel_epoch(FinalizeChannelEpochArgs {
            channel_epoch,
            epoch: epoch(),
            now,
        })
    }

    #[test]
    fn freezes_shares_for_claims() {
        let deposited = first_deposit(channel_epoch(), START, 100).unwrap();
        let channel_epoch_id = deposited.channel_epoch_next.id();

        let finalized = finalize(deposited.channel_epoch_next, START + DURATION).unwrap();

        assert_eq!(finalized.channel_epoch_next.clearing_credits_balance, 100);
        assert_eq!(
            finalized.channel_epoch_next.weighted_shares_balance,
            shares(200)
        );

        let channel_epoch_final = &finalized.channel_epoch_final;
        assert!(channel_epoch_final.channel_epoch_id == channel_epoch_id);
        assert_eq!(channel_epoch_final.unclaimed_weighted_shares, shares(200));
    }

    #[test]
    fn rejects_finalize_before_epoch_end() {
        let result = finalize(channel_epoch(), START + DURATION - 1);
        assert_eq!(result.err(), Some(E::EpochNotEnded));
    }
}
