# Referral kernel migration plan

Review and implementation record · 2026-11-08

The kernel migration is implemented. The pre-migration observations and staged
plan below remain as design history; current APIs and numerical rules are
colocated in [src/lib.rs](src/lib.rs) and [src/program.rs](src/program.rs).
`set_balance` replaces separate deposit/withdraw actions. Emission is an external
cumulative snapshot, and family stake/sharing actions own and return their
protocol and source-lane records. No unit tests were added; validation uses builds,
linting, the executable example, and separate runtime/accounting probes.

The kernel's maximum depth is now a network parameter, not a fixed depth of
seven. The `u8` depth representation permits `H` from 0 through 255, and terms
permit zero commission subject to `H*q < 1`. The 16-lane capacity comes from the
current `u16` lane mask. References below to eight-member paths (`H = 7`) describe
the original example and validation profile, not a kernel height restriction.
Larger configured heights require their own host resource validation.

One correction to the original projection plan is essential: eligibility changes
must also close upstream gauge/lane residuals, or old dust can reach newcomers.
Those actions commit the shared records; permissionless settlement may still
project read-only. This creates a global protocol write on balance and family
changes. Host transaction throughput and custody integration remain deployment
work, separate from the kernel's bounded memory.

## Completed kernel validation

The workspace check, strict crate Clippy, formatting, and warning-free Rustdoc
pass. The executable example collects exactly 110, 107.5, and 82.5 credits.
Separate temporary public-action probes checked sixteen-lane late staking,
withdrawals, prospective sharing, omitted sources, cadence, and old dust. A
500-event comparison against an independent rational oracle reconciled all
liabilities, carries, and unallocated amounts exactly; recipient error stayed
below two billionths of a credit base unit.

Fourteen compiled SBF scenarios passed in LiteSVM, retaining every complete
receipt and comparing native/SBF results. With `U192F64` and 32-byte IDs, the
measured heap watermark was 3,416 bytes for eight-member paths and 3,968 bytes
for sixteen-source families. Sharing and escrow changes used approximately
253,000–278,000 compute units, including fixtures and measurement output but
excluding real account decoding, persistence, and token/AVM CPIs. Small and
near-`u64::MAX` balances were exercised; these are measured cases, not universal
worst-case compute bounds.

The combined probe dispatcher exceeded the stack limit. Separate non-inlined
instruction handlers passed without changing the kernel. Host adapters must
preserve that separation and request a sufficient compute budget; a larger heap
does not fix an oversized stack frame. The runtime probes used platform-tools
v1.58 and LiteSVM 0.15.2. They remain external validation artifacts, not unit tests
added to this library.

## 1. Recommendation and evidence boundary

Implement the [economic specification](SPEC.md) by retaining the pure public
`Args -> Result<Receipt>` action pattern and the existing separation between
member records and member/lane records. Put the new validation and settlement
machinery behind those actions. No caller should have to calculate commissions,
advance a family index, or remember to perform a prerequisite settlement.

The original review used **16 lanes, eight earning levels (`H = 7`), and 100,000
or more members** as an example deployment profile:

- Asset changes process one lane along at most `H + 1` members: eight in that
  profile.
- Marketing and family-escrow changes process at most 16 lane sources of one
  parent. They do not process an eight-by-sixteen ancestry matrix.
- Already-accrued collection touches one member, plus the host's mint accounts.
- No action enumerates siblings, descendants, or the network's member registry.

This establishes a feasible bounded architecture, not a completed Solana
performance certification. The revised actions, numeric representation, account
adapter, and custody CPIs must pass the SBF resource gates below before release.
Native measurements of the existing kernel are reported separately from proposed
bounds. Account contention also limits throughput independently of memory.

**Depth convention:** a path contains at most `H + 1` earning members. “Eight
levels, including the member and up to seven ancestors” therefore means `H = 7`.
The non-earning protocol root is not another path member. Configuring
`max_depth = 8` instead permits nine earning members; neither value is built in.

## 2. Pre-migration foundations and gaps

Useful foundations to preserve:

- [`lib.rs`](src/lib.rs) is `no_std`, forbids unsafe code, and exposes records
  and the [`program`](src/program.rs) action interface.
- [`LaneChain`](src/tree/lane_chain.rs) validates ancestry and settles a bounded
  path iteratively. Its variable-sized records live in `Vec` allocations.
- [`Protocol::gauge_at`](src/protocol.rs) and
  [`Lane::reward_index`](src/lane.rs) project current indices without requiring
  global state writes for every asset action.
- [`LanePool`](src/tree/lane_pool.rs) and the weighting layer already separate
  emission history, lane allocation, and reward-per-asset history.
- [`collect_credits`](src/program/collect_credits.rs) collects accrued credits
  once and leaves the fractional claim with the member.

The pre-migration gaps were:

| Pre-migration code                                                                        | Required migration                                                                                              |
| ----------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `LaneNode::settle` shares all core earnings by direct-child asset balances                | Separate own income from gross overrides; share only overrides through common credit escrow                     |
| `sharing_seen` activates a policy at each lane's next settlement                          | Close every parent source at one effective time before changing family terms                                    |
| `register_lane` takes no protocol state and records have no lane identity                 | Add an authenticated registry with 16 stable slots and keyed record associations                                |
| `LaneChain::new` treats omitted lane records as new zero state                            | Require records for every materialized lane; accept absence only for a legitimately new record                  |
| Depth is derived afresh from ancestry                                                     | Cache immutable depth at admission and validate it against the path when a path is supplied                     |
| Join price acceptance and owner authorization are host duties                             | Enforce semantic authorization and fee limits in actions using authenticated host facts                         |
| Top-level admission is an ordinary editable fee paid to the protocol                      | Consume a scarce auction allocation once and return a winning-payment burn effect                               |
| `Emitter` uses a changeable constant rate and saturates cumulative emission at `u64::MAX` | Consume the actual cumulative declining-emission schedule; separate cumulative emission from outstanding supply |
| `Change::apply` can panic on invalid totals; backward time is silently clamped            | Return explicit action errors for invalid amounts, chronology, and arithmetic                                   |
| Gauge/share remainders can be discarded or assigned by settlement cadence                 | Define explicit remainder ownership and conservation before choosing final state fields                         |

The baseline `cargo test -p mayf-referral --locked --offline` succeeds with
**zero unit tests and zero doctests**. The pre-migration `emissions` example succeeded
and checks the old payout vector `[1999999999, 4499999999, 3499999999]`. It is a
compilation/example baseline, not a test of the revised economics or custody.

## 3. State layout and bounded input validation

Keep small independently addressable records. Do not put all members in one
account, or all 16 asset positions into every member value passed on every
ordinary asset update.

| Record                 | Proposed additions or retained responsibilities                                                                                                                                                                                                                               |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Protocol/configuration | Network identity; fixed `H` and `q`; at most 16 permanent lane slots; emission/gauge configuration; root-admission authority or references                                                                                                                                    |
| Lane                   | Network/slot identity; gauge allocation checkpoint; eligible total; reward index; explicit rounding/unallocated accounting                                                                                                                                                    |
| Node                   | Identity and authority; immutable parent and earning depth; offer; current marketing fraction; own escrow `s`; total child escrow `S`; outgoing family index `F`; incoming checkpoint; net claimable credits; lane mask; time high-water mark; required fractional accounting |
| LaneNode               | Member/network/lane association; own assets `p`; strict descendant assets `D`; lane index checkpoint; required fractional accounting                                                                                                                                          |
| Auction allocation     | Network, unique allocation identity, winner, seat authorization, price, and consumed status                                                                                                                                                                                   |
| Custody position       | Issued-credit backing for the authenticated member/family or gauge position, interpreted by the host adapter                                                                                                                                                                  |

Use `LaneId`/slot values in `0..16`. Registration rejects duplicate asset/lane
identities, a seventeenth lane, and reused slots. Permanent slots make historical
checkpoints unambiguous. The host binds the registered identity to the actual
eligible asset and canonical account addresses.

Each node keeps an **ever-materialized `u16` lane mask**. Set its bit when that
node's own or descendant position first creates a LaneNode. Keep the bit after
balances return to zero. A family action requires exactly the sources named by
that authenticated mask; the caller cannot omit a stale or unfavorable lane.
This bounds source discovery without scanning the tree or maintaining an
unbounded list. Missing records for set bits are errors.

On first activation, start the new LaneNode at the current projected lane index
before adding assets. Update the ancestor masks in the same action as their
descendant totals. A lane newly registered later does not require touching every
node: its bit is absent until each relevant position first activates.

Actions validate lengths before traversal, duplicate keys, network/slot/member
associations, complete parent links, cached depth consistency, permissible depth,
effective represented `H*q < 1`, amounts, and time. Zero depth and zero
commission are valid generic configurations. A supplied path must terminate at
the correct depth-zero seat. A registered maximum
is an input limit, not permission to truncate an overlong path.

With cached depth, a family change can validate the authenticated immediate
parent/child relationship without loading the parent's ancestors. Admission and
asset-changing actions still validate their complete required paths. Ordinary
admission needs no family barrier because the new child starts with zero escrow.

### Host facts versus kernel rules

The action checks the actor against the record's authority and validates every
economic/state transition. The host authenticates the facts the pure library
cannot discover: signatures, account owners and addresses, canonical absence,
clock, token balances, external auction results, and eligible asset positions.
A forged public struct cannot prove its own on-chain provenance.

Represent these facts explicitly in keyed inputs/action context. Do not move
commission arithmetic or sequencing into the host. The host supplies facts,
invokes one public action, and applies its receipt atomically.

## 4. Private settlement machinery

### 4.1 Asset changes remain one-lane path operations

For a deposit or withdrawal at effective time `t`:

1. Validate the action, complete path, source identities, and available balance.
2. Project the cumulative emitter and gauge to `t`; advance the selected lane
   using its old balance and allocation history.
3. Iterate from the earning seat toward the member. For each LaneNode, compute
   own income and gross overrides separately from `delta_I`.
4. Route that node's override marketing through its current `mu`, `S`, and global
   family index. Materialize incoming family gifts as appropriate, without
   feeding those gifts into outgoing marketing.
5. Apply the asset delta to the member, lane total, and each ancestor's strict
   descendant total. Initialize any new source records at the current index.
6. Return the same bounded record buffers plus the required asset/custody effects.

No sibling loads are necessary. Unchanged sibling escrow balances later multiply
the family index growth. Credit escrow does not change with an asset trade.

### 4.2 Family changes close only that parent's sources

Implement one private family-settlement operation and call it inside
`set_marketing`, `stake_family_credits`, and `unstake_family_credits`:

1. Validate the parent's complete source mask and keyed lane records.
2. Use one trusted `t` and project the gauge once.
3. For every required lane, project its reward index and accrue the parent's
   own income and gross overrides under the old family state.
4. Account for the parent's net earnings and publish the family index increment
   under the old `mu` and `S`.
5. For an escrow mutation, settle the affected child's incoming entitlement under
   its old `s`, then change child `s` and parent `S` and return the custody effect.
   For a marketing mutation, install the new `mu` after the barrier.

Incoming gifts are terminal, so this operation does **not** need to settle the
parent's incoming gifts or recurse into its ancestors' other lanes. Other
children's checkpoints remain lazy. No multi-transaction freeze or economic
lock period is needed if this bounded action meets the resource gate.

Within a barrier, accumulate the parent's lane deltas and divide the combined
marketing amount by `S` once where the rounding design permits. The common
family carry must make this equivalent to earlier one-lane settlements; batching
must not change who owns fractional rewards.

### 4.3 Projection and eligibility boundaries

Permissionless source and family settlement can project global indices read-only.
A later canonical update must commit at `T >= t` before changing its weights or
balances, and must not produce an index below an earlier projection.

Actual asset changes additionally commit the Gauge and close its shared residue.
Actual marketing or escrow changes commit the Gauge and every parent source Lane,
then close their shared residuals and the parent's marketing/family residuals
before changing eligibility. Closing only the parent carry is insufficient: old
upstream dust could otherwise arrive under new terms. These closures are bounded
unallocated losses, and private fractional earnings remain owned.

Read-only projections never add their observed unallocated amounts to a stored
ledger. Only canonical commits do that, preventing duplicate loss accounting.

### 4.4 Collection has an honest scope

Keep `collect_credits` small: collect already-materialized net entitlement and
emit a typed mint effect. It does not promise to settle every source through now.

A separate public action may settle a family and materialize one child's incoming
claim. Clients can use public settlement actions before collection; they never
implement the settlement arithmetic themselves. An optional atomic
`collect_through(now)` needs the member's own sources and its immediate parent's
sources, including when the member has no asset records. It still needs no full
ancestry matrix, but its larger account set must be measured separately. Do not
make that convenience action a prerequisite for simple collection.

## 5. Action working sets

For the separate-record layout, let `D <= H + 1` be path members and `L <= 16`
the relevant source-lane count. The original `H = 7` profile has `D <= 8`.
These counts exclude signer, program, token, auction, and account-creation
accounts added by the host.

| Action                       | Read-only kernel records            | Written kernel records                                        | Maximum distinct kernel records |
| ---------------------------- | ----------------------------------- | ------------------------------------------------------------- | ------------------------------: |
| Reconcile one asset balance  | —                                   | Protocol, 1 Lane, D Nodes, D LaneNodes                        |          `2 + 2D` (18 at `H=7`) |
| Set marketing fraction       | —                                   | Protocol, L Lanes, 1 parent Node, L parent LaneNodes          |                              34 |
| Stake/unstake family credits | —                                   | Protocol, L Lanes, parent and child Nodes, L parent LaneNodes |                              35 |
| Set offer                    | Relevant authenticated context      | 1 Node                                                        |                        Constant |
| Collect accrued credits      | Relevant authenticated context      | 1 Node                                                        |                        Constant |
| Optional collect through now | Protocol, union of at most 16 Lanes | Member and parent Nodes, at most 32 LaneNodes                 |                              51 |

The table assumes authority and registry information remains in the listed
records; splitting that information into extra accounts increases the count.
The Solana wrapper must publish actual read/write account manifests, not merely
copy these logical counts. In particular, compose the referral call with the
real AVM and token actions when evaluating transaction limits.

### Storage and contention

At 100,000 members and all 16 lanes populated, separate storage can reach
1.6 million LaneNode records plus 100,000 Node records. Typical sparse creation
reduces that count. None of these records is loaded globally, but persistent
account storage, funding, RPC indexing, and transaction volume still cost real
resources. There is no protocol-wide member vector or fixed 100,000-member cap.

The principal throughput limits remain:

- Asset and family eligibility changes write the shared Protocol gauge.
- Asset changes in one lane write the same Lane total.
- Changes under a shared earning seat write shared ancestor Nodes, including
  their claimable credits and common family indices, even across different lanes.
- A 16-source family eligibility change writes all 16 source Lane accounts.
  Read-only family settlement also conflicts with a concurrent writer. [Agave account locking](https://github.com/anza-xyz/agave/blob/master/accounts-db/src/account_locks.rs)
- Credit collection mints through a common credit mint account.

Keep permissionless settlement read-only on Protocol and global lanes. Actual
eligibility changes commit them to close old shared dust. Avoid adding a single
global escrow vault as another unnecessary write hotspot; the host can
use custody scoped to a family/member. Measure realistic same-family and
different-family traffic. Population capacity does not establish a transactions-
per-second guarantee.

If measured contention is unacceptable, revisit per-lane family accounting or
versioned settlement as a separate, equivalence-proven optimization. Packing
records alone does not remove these dependencies. Do not add epochs, cooldowns,
or delayed marketing activation as an unannounced performance workaround.

## 6. Solana memory and transaction design

### Runtime budgets

Solana documents a **4,096-byte stack frame**, a **32 KiB default heap**, and an
adjustable heap up to **256 KiB**. These are distinct limits: a larger heap does
not enlarge a stack frame. [Program limits](https://solana.com/docs/core/programs)

Budget the initial adapter for the conservative legacy/v0 limits: **64 accounts**
and **1,232 transaction bytes**, using v0 address lookup tables for larger family
actions. Current v1 transactions support **4,096 wire bytes**, but use a different
message configuration and no address lookup tables. They do not change the SBF
stack-frame limit. Validate the transaction version and tools actually used by
the host. [Transaction limits](https://solana.com/docs/core/transactions),
[v1 format](https://solana.com/upgrades/larger-transaction-sizes)

The documented compute ceiling is **1.4 million CUs per transaction**. The wrapper
must measure and set its budget, including CPIs; the pure library's operation
count is not a CU measurement. [Compute budget](https://solana.com/docs/core/fees/compute-budget)

### Existing native measurements

Fresh native builds with `[u8; 32]` identifiers produced these `size_of` values:

| Pre-migration type                            | U64F64 backend | U192F64 backend |
| --------------------------------------------- | -------------: | --------------: |
| Node                                          |          112 B |           120 B |
| LaneNode                                      |           80 B |           128 B |
| Lane                                          |           80 B |           112 B |
| Protocol                                      |           96 B |           104 B |
| Deposit args, including vector descriptors    |          352 B |           400 B |
| Deposit receipt, including vector descriptors |          240 B |           280 B |

A native allocation probe of an actual depth-7 deposit, including input vectors
and receipt construction, measured:

| Measurement                | U64F64 backend | U192F64 backend |
| -------------------------- | -------------: | --------------: |
| Heap allocations           |              5 |               5 |
| Cumulative requested bytes |        5,136 B |         6,408 B |
| Peak live requested bytes  |        3,456 B |         4,288 B |

These are pre-migration native measurements, not migrated-kernel sizes, SBF frame
measurements, serialized account sizes, or host/CPI memory measurements. The
default SBF bump allocator does not reclaim ordinary deallocations, and native
alignment can differ from SBF. Budget cumulative allocation and alignment,
not just peak live native bytes. [Entrypoint allocator](https://github.com/anza-xyz/solana-sdk/blob/master/program-entrypoint/src/lib.rs)

### Implementation rules

1. Keep `Vec`/heap-backed, length-validated buffers for paths and source sets.
   Large inline arrays of records must not appear in action arguments, receipts,
   or local variables. Eight-by-sixteen pre-migration LaneNodes alone would occupy
   10,240 or 16,384 native bytes, depending on backend.
2. Consume an owned working buffer and return it in the receipt where possible.
   Replace the pre-migration input-vector -> Link-vector -> output-vectors allocation
   sequence with one bounded working representation. Read-only data may be
   borrowed; purity does not require cloning it.
3. Allocate final capacity once, avoid vector growth, and reuse capacity. Do not
   assume dropping a vector recovers SBF heap during the instruction.
4. Construct large objects directly in their heap buffers. `Box::new(large_value)`
   by itself does not establish that the compiler avoids a large stack temporary.
5. Process one source/record at a time, with iterative loops and small numeric
   temporaries. Do not recurse along ancestry or serialize entire receipts into
   stack arrays. A Rust receipt is an internal return value, not Solana CPI
   return data or a transaction payload containing all state. Ancestor/source
   iteration stays inside the kernel call; it is not one CPI per member or lane.
6. Keep account serialization in the host. Use a checked compact layout with
   explicit decoding; the kernel retains `forbid(unsafe_code)` and no dependency
   on Solana's account types. Parse only authenticated bounded account sets.
7. Start with separate LaneNode accounts. Packing all lanes into each member
   reduces account count but makes ordinary one-lane ancestry updates load much
   more data. Consider packing only after measuring a specific bottleneck.
8. Use a shared lookup table for stable Protocol/lane addresses where useful.
   This may be enough for a 35-kernel-account family transaction without a
   separate table per family. Serialize the real transaction to verify it.

A requested larger heap is a host fallback to measure, not a substitute for
bounded buffers. Its allocator must actually use the requested heap region.
Keep sufficient headroom for instruction decoding, account metadata, numeric
helpers, serialization, and nested token/AVM calls.

## 7. Emission, precision, and receipts

### Cumulative emission interface

Keep emission as a separate pure component behind actions. The referral layers
need authenticated cumulative scheduled emission `B(t)` and its historical
checkpoints, with `B(b)-B(a) = integral_a^b E(t) dt` for the planned continuously
declining `E(t)`. Evaluating this must not loop over every elapsed second.

The exact curve remains an emitter-design dependency. It can be implemented by
the internal pure emitter or an explicitly trusted adapter; an untrusted caller
cannot supply an arbitrary larger `B(t)`. Gauge and asset changes preserve old
allocation intervals, so the current index layering can remain.

The pre-migration arbitrary `set_emission_rate` operation did not define a
continuous-decay policy. That API and its saturating `rate * elapsed`
implementation were replaced by the authenticated cumulative-emission interface.
The actual decay curve remains the external emitter's responsibility.

### Numeric design comes before final record sizes

Retain whole token base units for transfers and the fixed-number abstraction for
fractional accounting. Validate the selected concrete host backend against the
new operations; do not assume the old `create_protocol` check proves new bounds.

Cumulative scheduled emission, outstanding token supply, and reward-per-unit
indices are different quantities. Auction burns make the distinction especially
important. The pre-migration proof that everything fits below `2^64` relies on
stopping cumulative emission at that value. It cannot be reused as a proof of
the new model. The far-future outstanding-supply policy remains deferred; normal
checked arithmetic and validated operational bounds remain necessary now.

Before freezing the state layout, specify:

- A single represented `q` used consistently for holder deductions and ancestor
  commissions, with its effective depth bound checked.
- Fractional own-income, override, family-index, and child-claim accounting.
- Carries or a proven error bound that does not make settlement cadence a way to
  redirect material rewards.
- Ownership of shared residuals when eligibility changes, including a change of
  owners with the same denominator. Explicit unallocated accounting is permitted
  where the economic spec permits it; silently gifting residuals to newcomers is
  not. Individual earned fractions remain individual claims.
- Monotone projected indices and equivalence between batched and one-source
  settlement.
- Separate unallocated totals and outstanding marketed liabilities, without
  giving both parent and children a claim on the same amount.

If current `FixedNumber` operations cannot expose the precision needed for this
proof, add the smallest required checked wide/intermediate or remainder helper.
Do not route a production action through an arbitrary-precision reference type.
Measure the concrete representation after these requirements are established.

### Typed economic effects

Each action receipt should identify both changed records and its required
effects: asset deposit/withdrawal or position update, join-fee transfer, escrow
deposit/return, winning-payment burn, or collected-credit mint. Include network,
member/recipient, asset identity, and amount as appropriate; an unbound amount is
not a complete receipt contract.

The kernel mutates owned drafts and returns no receipt on failure. The host
commits every record and custody effect together, or none. On Solana, propagate
failed token/account CPIs so the transaction rolls back; do not catch the failure
and commit the kernel draft. Desktop/test hosts need an equivalent transaction
boundary. Authenticate fresh state at application time, including replay-safe
auction consumption. [Transaction atomicity](https://solana.com/docs/core/transactions)

## 8. Implementation sequence and acceptance gates

### Phase 1 — Freeze bounds and establish executable references

- Record 16 permanent lane slots and use eight earning levels (`H=7`) as an
  example validation profile. Keep `H` and numerical `q` configurable network
  terms satisfying `H*q < 1`, not additional parent settings. Validate any
  deployment's chosen height separately; the generic kernel also accepts zero
  depth or zero commission.
- Create an independent exact-rational, event-driven oracle and a transactional
  in-memory host exercising public actions. Cover core and marketing
  conservation, late staking, terminal gifts, source separation, changing
  balances, declining emission, fee transfers, burns, and escrow principal.
- Add boundary fixtures for all lanes and the deepest path. Add a Solana host
  harness early, using the intended concrete number types and actual action
  monomorphizations. A generic-library build alone is insufficient.
- Pin compatible Rust/SBF toolchains. The inspected workspace uses Rust 1.97.1;
  installed build-sbf reports 3.1.10/platform-tools 1.52 with Rust 1.89.0. Resolve
  actual compatibility in the harness rather than assuming a native build proves it.

**Gate:** reproducible oracle and real SBF harness, with old behavior clearly
separated from the new acceptance vectors.

### Phase 2 — Migrate identities, bounded state, and errors

Change `protocol.rs`, `lane.rs`, `tree/node.rs`, `tree/terms.rs`,
`program/register_lane.rs`, `numbers.rs`, and `error.rs`:

- Add lane registry, keyed inputs, cached depth, lane masks, authority context,
  and time validation.
- Reject overlong paths, duplicate or missing sources, wrong identities, invalid
  represented terms, backdating, and arithmetic failures through typed errors.
- Decide the carry representation and test its invariants before finalizing
  serialized field layouts.

**Gate:** malformed input never yields a partial receipt; valid input size is
independent of total network population.

### Phase 3 — Migrate core accrual and emitter integration

Change `emission/*`, `weighting/*`, `tree/lane_pool.rs`, `tree/lane_node.rs`, and
`tree/lane_chain.rs`:

- Use cumulative scheduled emission, preserving lazy gauge/lane projections.
- Separate own income from overrides; remove `children`, lane-local gift indices,
  and `sharing_seen` once the global family machinery replaces their use.
- Replace allocation-heavy chain reconstruction with bounded reusable buffers.
- Implement explicit unallocated and fractional accounting.

**Gate:** oracle-equivalent one-lane path operations and monotone read-only
projections through deposits, withdrawals, gauge changes, and empty intervals.

### Phase 4 — Add the family state machine

- Add private complete-source validation and common-time settlement.
- Implement self-contained public marketing, family stake, family withdrawal,
  and incoming-gift settlement actions; each performs every prerequisite itself.
- Preserve zero-escrow behavior, initial checkpoints, and earned fractions on
  complete withdrawal. Credit custody is separate from asset eligibility.

**Gate:** all 16-lane late-entry, withdrawal, policy-change, omitted-source,
zero-asset recipient, batching, and same-timestamp cases pass the oracle.

### Phase 5 — Migrate admission, collection, and host effects

- Replace unrestricted `join_at_top_level`/`set_top_level_offer` with authorized
  seat activation and one-time burn consumption. Keep the auction algorithm
  external to the referral mechanism.
- Put ordinary join fee limits and exact successful-join increments in the join
  action. Initialize empty child state correctly without loading every lane.
- Preserve simple accrued collection; add larger complete-settlement convenience
  only if it passes the host budget.
- Update `program.rs`, root exports, the example host, and crate documentation.
  Maintain one clear public action interface rather than legacy and new economics
  coexisting behind ambiguous names.

**Gate:** failing any transfer, escrow movement, burn, mint, or late arithmetic
step leaves the transactional host unchanged; repeats cannot spend or mint twice.

### Phase 6 — Certify the actual resource and traffic envelope

Measure the final wrapper, not only isolated arithmetic:

| Scenario                                                     | Required evidence                                                                                 |
| ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------- |
| Warm deposit/withdraw at configured maximum depth            | Account manifest, serialized transaction, SBF frame use, cumulative heap, CUs                     |
| First deposit in a new lane                                  | Same measurements including creation of up to `H + 1` LaneNode accounts, funding and custody CPIs |
| Parent marketing change with 16 occupied sources             | Complete barrier in one transaction; prior/new ownership matches oracle                           |
| Stake/unstake with 16 sources                                | Above plus actual issued-credit custody and rollback                                              |
| Zero-asset child's collection                                | Correct multi-lane gifts; separately measure simple and optional complete collection              |
| AVM buy/sell composed with referral update                   | Total transaction/CPI account, compute, heap, and instruction budgets                             |
| 100,000-member simulated host                                | Record access counts remain bounded by the table, including wide sibling sets                     |
| Concurrent same-branch, cross-branch, and cross-lane traffic | Lock contention, success rate, and latency at the intended workload                               |

Reject stack-frame overruns, heap exhaustion, transaction-size/account-limit
violations, excessive CPI nesting or instruction counts, and compute overruns.
Measure the complete AVM/referral/token call graph against the target runtime's
CPI-depth limit. Require headroom for the host's full call path. Do not label the
design Solana-validated until these results exist.

Finish with focused tests, the relevant workspace checks, and documentation that
describes the implemented actions rather than leaving the target/current split
unresolved. Conversion of any durable deployed state requires a separate
explicit rollout; this plan is the library and adapter migration, not evidence
that existing live account schemas can be reinterpreted safely.
