# mayf-spl-token

Small effectful helpers for Anchor programs using the original SPL Token program:
`transfer`, `burn`, `close_account`, and `set_authority`. Calling programs decide
what should move; this crate only handles the CPI.

Each function takes ordinary `AccountInfo` handles and borrowed signer seed
sets. Pass `&[]` for an instruction signer, or `&[&seeds]` for a PDA authority:

```rust
mayf_spl_token::transfer(
    ctx.accounts.source.to_account_info(),
    ctx.accounts.destination.to_account_info(),
    ctx.accounts.authority.to_account_info(),
    amount,
    &[&seeds],
)?;
```

Zero transfers and burns return successfully without invoking SPL Token. They
perform no token validation. Nonzero movements forward SPL errors unchanged.
Account handles share the original storage; these helpers neither deserialize
nor clone account payloads, and they do not allocate or copy signer seeds.

The helpers use SPL Token's fixed program ID and single-authority instructions.
Callers still supply the executable Token program in the transaction and
authenticate their instruction accounts with Anchor. Token-2022 and SPL
multisig authorities require their respective interfaces.

Host tests check zero-movement and account-borrow behavior. Run them from the
workspace root:

```sh
cargo test -p mayf-spl-token --locked
```
