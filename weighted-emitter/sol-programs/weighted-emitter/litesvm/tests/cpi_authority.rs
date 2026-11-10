use mayf_weighted_emitter_test_caller as caller;

use crate::support::*;

#[test]
fn a_composing_program_can_create_and_settle_through_its_pda_authority() {
    let mut f = Fixture::empty(false);
    let authority = caller::authority().0;
    let mut create = f.emitter_ix();
    meta_mut(&mut create, f.admin.pubkey()).pubkey = authority;
    let mut gauge = f.gauge_ix(&f.gauge);
    meta_mut(&mut gauge, f.admin.pubkey()).pubkey = authority;
    f.bank.send(
        &[caller::wrap(create), caller::wrap(gauge)],
        &[&f.emitter, &f.gauge],
    );
    f.bank.send(
        &[f.position_ix(&f.voter), f.deposit_ix(&f.voter, 100)],
        &[&f.voter.authority],
    );
    assert_eq!(f.state().0.authority, authority);

    f.bank.set_time(START + 10);
    let mut settle = f.settle_ix();
    meta_mut(&mut settle, f.admin.pubkey()).pubkey = authority;
    let mut unsigned = settle.clone();
    meta_mut(&mut unsigned, authority).is_signer = false;
    f.bank.reject(&[unsigned], &[], ix_error(0, 3010));
    let minting = f.minting_snapshot(f.gauge.pubkey(), f.destination);
    f.bank.send(&[caller::wrap(settle)], &[]);
    minting.assert_minted(&f.bank);
}
