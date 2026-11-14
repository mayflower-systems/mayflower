# CLAUDE.md

## Design docs

`docs/github-wiki/content/` holds the Mayflower design docs: the AVM, credits
and emissions, the referral network, and revenue clearing. Read the relevant
pages before you change protocol logic. Start at `Home.md` in that folder.
`_Sidebar.md` lists every page.

The wiki is a leaf. Its pages are self-contained and link to other wiki pages
by name. The link `[x](AVM-Floor-Raising)` points to `AVM-Floor-Raising.md`
wherever it sits under `content/`. Repository docs can link into the wiki
using relative paths.

The folder is a git subtree of the GitHub wiki. `docs/github-wiki/README.md`
explains how to sync it. `git subtree push` publishes to the live wiki, so run
it only when asked.

## Documentation

Keep each module's documentation within its dependency boundary. Explain how
modules work together where they are composed.

Keep READMEs focused on the current design, the problem it solves, how to use
it, and where to start reading the code. Omit package inventories and obsolete
design history. Document API contracts, invariants, and edge cases in the
owning code's docstrings. Check the current implementation when correcting
documentation drift.

## State-machine boundaries

Public state-machine actions own business validation and state transitions,
including lifecycle rules, timing, disabled states, and domain relationships.
Anchor handlers convert accounts to library models, call an action, map its
domain errors to explicit Anchor error codes, perform side effects, and persist
the receipt. Anchor constraints belong to account ownership, authorization,
PDA creation, and token custody; do not put business rules in them. Validate
existing accounts through stored relationships rather than rederiving their PDA
seeds. Domain foreign keys belong to the library; authority checks stay in Anchor.
Do not duplicate SPL Token's mint or transfer/burn authorization checks in Anchor
constraints. A signer may choose any refund destination owner. Keep the stored
escrow address checks and the mint/authority settings needed to initialize escrows.

## Tests

Keep tests focused on nontrivial algorithms, state transitions, and meaningful
invariants. Do not add tests for mechanical `Into`/`From` conversions, simple
field mappings, or routine serialization round trips. Fixed-point arithmetic
and rounding tests belong in the fixed-number crates that own those operations,
not in every consumer. Avoid duplicating library tests in host adapters.
