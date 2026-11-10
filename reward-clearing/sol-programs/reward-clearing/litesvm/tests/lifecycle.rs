use crate::support::*;
use cpi::{
    BidderPosition, Channel, ChannelEpoch, ChannelGroup, Epoch, InstructionArgs,
    MayfRewardClearingError, Organization,
};

fn assert_empty_vault(bank: &Bank, address: Address, mint: Address, authority: Address) {
    assert_eq!(
        bank.account(&address).owner,
        cpi::instructions::TOKEN_PROGRAM_ID
    );
    bank.assert_rent_exempt(&address);
    let token = bank.tokens(&address);
    assert_eq!(
        (token.mint, token.owner, token.amount),
        (mint, authority, 0)
    );
}

#[test]
fn initialization_persists_account_relationships_and_rent_exempt_custody() {
    let f = Fixture::new();
    let a = f.addresses;
    for address in [
        a.organization,
        a.group,
        a.epoch,
        a.channel,
        a.channel_epoch,
        a.position,
    ] {
        assert_eq!(f.bank.account(&address).owner, PROGRAM_ID);
        f.bank.assert_rent_exempt(&address);
    }
    assert_eq!(
        f.bank.read::<Organization>(&a.organization),
        Organization {
            authority: f.admin.pubkey(),
            next_channel_group_index: 1
        }
    );
    let group = f.bank.read::<ChannelGroup>(&a.group);
    assert_eq!(
        (group.organization, group.clearing_credits_mint),
        (a.organization, a.credit_mint)
    );
    assert_eq!(
        (group.channel_group_index, group.current_epoch_index),
        (0, 0)
    );
    assert_eq!(group.master_start_time_unix_seconds, START as u64);
    assert_eq!(group.epoch_duration_seconds, DURATION);
    assert_eq!(
        f.bank.read::<Epoch>(&a.epoch),
        Epoch {
            channel_group: a.group,
            epoch_counter: 0,
            start_time_unix_seconds: START as u64,
            duration_seconds: DURATION,
        }
    );
    let channel = f.bank.read::<Channel>(&a.channel);
    assert_eq!(channel.channel_group, a.group);
    assert_eq!(channel.reward_token_mint, a.reward_mint);
    assert_eq!(channel.reward_staging_token_account, a.staging);
    assert_eq!(channel.default_epoch_terms, TERMS);
    assert_eq!(channel.last_created_epoch_counter, 0);
    assert!(!channel.disabled);
    let epoch = f.bank.read::<ChannelEpoch>(&a.channel_epoch);
    assert_eq!((epoch.channel, epoch.epoch_counter), (a.channel, 0));
    assert_eq!(epoch.reward_pot_escrow_token_account, a.reward_pot);
    assert_eq!(epoch.clearing_credits_escrow_token_account, a.credits);
    assert_eq!(epoch.clearing_credits_balance, 0);
    assert_eq!(epoch.reward_pot_balance, 0);
    assert_eq!(epoch.weighted_shares_balance_q64f64, 0);
    assert_eq!(
        epoch.terms,
        cpi::StoredEpochTerms {
            ramp_bonus_slope_q64f64: 0,
            exit_penalty_exponent: 1
        }
    );
    assert_eq!(epoch.final_totals, None);
    let position = f.bank.read::<BidderPosition>(&a.position);
    assert_eq!(
        (position.authority, position.channel, position.epoch_counter),
        (f.bidder.pubkey(), a.channel, 0)
    );
    assert!(!position.claimed);
    assert!(position.lots.is_empty());
    assert_eq!(f.bank.account(&a.position).data, position.to_account_data());
    for (address, mint, authority) in [
        (a.group_credits, a.credit_mint, a.group),
        (a.staging, a.reward_mint, a.channel),
        (a.reward_pot, a.reward_mint, a.channel_epoch),
        (a.credits, a.credit_mint, a.channel_epoch),
    ] {
        assert_empty_vault(&f.bank, address, mint, authority);
    }
}

#[test]
fn administration_preserves_open_epoch_and_permissionless_creation_uses_new_defaults() {
    let mut f = Fixture::new();
    let a = f.addresses;
    let preserved = Snapshot::capture(
        &f.bank,
        [a.organization, a.epoch, a.channel_epoch, a.position],
    );
    let terms = cpi::EpochTerms {
        exit_penalty_exponent: 2,
        ..TERMS
    };
    let mut channel = f.bank.read::<Channel>(&a.channel);
    f.bank
        .send(&[f.terms_ix(terms), f.disable_ix(true)], &[&f.admin]);
    channel.default_epoch_terms = terms;
    channel.disabled = true;
    assert_eq!(f.bank.read::<Channel>(&a.channel), channel);
    f.bank.send(&[f.disable_ix(false)], &[&f.admin]);
    channel.disabled = false;
    assert_eq!(f.bank.read::<Channel>(&a.channel), channel);

    f.bank.set_time(START + i64::from(DURATION));
    let mut group = f.bank.read::<ChannelGroup>(&a.group);
    // Neither the organization authority nor the bidder signs this transition.
    f.bank.send(&[f.epoch_ix(1), f.channel_epoch_ix(1)], &[]);
    group.current_epoch_index = 1;
    channel.last_created_epoch_counter = 1;
    assert_eq!(f.bank.read::<ChannelGroup>(&a.group), group);
    assert_eq!(f.bank.read::<Channel>(&a.channel), channel);
    assert_eq!(
        f.bank.read::<Epoch>(&a.epoch_at(1)),
        Epoch {
            channel_group: a.group,
            epoch_counter: 1,
            start_time_unix_seconds: START as u64 + u64::from(DURATION),
            duration_seconds: DURATION,
        }
    );
    let next = f.bank.read::<ChannelEpoch>(&a.channel_epoch_at(1));
    assert_eq!((next.channel, next.epoch_counter), (a.channel, 1));
    assert_eq!(next.clearing_credits_balance, 0);
    assert_eq!(next.reward_pot_balance, 0);
    assert_eq!(next.weighted_shares_balance_q64f64, 0);
    assert_eq!(next.final_totals, None);
    assert_eq!(
        next.terms,
        cpi::StoredEpochTerms {
            ramp_bonus_slope_q64f64: 0,
            exit_penalty_exponent: 2
        }
    );
    assert_eq!(next.reward_pot_escrow_token_account, a.reward_pot_at(1));
    assert_eq!(next.clearing_credits_escrow_token_account, a.credits_at(1));
    for (vault, mint) in [
        (a.reward_pot_at(1), a.reward_mint),
        (a.credits_at(1), a.credit_mint),
    ] {
        assert_empty_vault(&f.bank, vault, mint, a.channel_epoch_at(1));
    }
    preserved.assert_unchanged(&f.bank);
}

#[test]
fn rejected_creation_restores_allocations_rent_and_any_earlier_instruction() {
    for packed in [false, true] {
        let mut f = Fixture::empty();
        if !packed {
            f.bank.send(&[f.organization_ix()], &[&f.admin]);
        }
        let mut invalid_group = f.group_ix();
        invalid_group.data = cpi::instructions::args::CreateChannelGroup {
            args: cpi::CreateChannelGroupArgs {
                master_start_time_unix_seconds: START as u64,
                epoch_duration_seconds: DURATION - 1,
            },
        }
        .to_instruction_data();
        let instructions = if packed {
            vec![f.organization_ix(), invalid_group]
        } else {
            vec![invalid_group]
        };
        f.bank.reject(
            &instructions,
            &[&f.admin],
            ix_error(
                u8::from(packed),
                MayfRewardClearingError::EpochDurationTooShort as u32,
            ),
        );
        let a = f.addresses;
        for address in [a.group, a.epoch, a.group_credits] {
            assert!(f.bank.svm.get_account(&address).is_none());
        }
        if packed {
            assert!(f.bank.svm.get_account(&a.organization).is_none());
        } else {
            assert_eq!(
                f.bank
                    .read::<Organization>(&a.organization)
                    .next_channel_group_index,
                0
            );
        }
    }
}
