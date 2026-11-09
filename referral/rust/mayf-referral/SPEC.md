# Referral Tree and Credit Mutualization

Implementation handoff · 2026-11-08

## 1. Status and scope

This document specifies the target economic model and its accounting semantics.
It supersedes the deleted `mutualization_idea.md` proposal. It is not a statement
of deployment readiness or a security audit. Section 13 maps the rules to the
migrated kernel; source documentation describes its numerical accounting.

The system allocates a scheduled global stream of credits to deposited assets and
their referral ancestors. Parents compete for children using credit-denominated
admission prices and discretionary marketing budgets. Children compete for those
budgets by escrowing credits with their immediate parent.

The layers are separate:

1. **Root admission:** the protocol auctions a small, scarce supply of top-level
   seats. The winning credit payment is burned.
2. **Lane allocation:** an external allocator, such as gauge voting, assigns
   portions of global emission to asset lanes.
3. **Core referral allocation:** fixed commissions divide each lane's allocation
   among asset holders and their ancestors.
4. **Mutualization:** a parent voluntarily shares some of its referral commissions
   with its immediate children, in proportion to their escrowed credits.
5. **Ordinary admission:** a child pays its parent's current join fee in credits.
   The payment belongs to the parent.

The planned global emission rate decreases continuously. The earlier fixed-rate
assumption simplified the allocation analysis; it is not the intended emission
policy. The auction format, gauge-voting mechanism, revenue-clearing mechanism,
and exact emission decay curve are specified separately from the tree. Their
interfaces and credit-custody boundaries are specified here. Allocation must
preserve the actual time-varying emission history.

**Mutualization is a marketing offer, not permanent income ownership.** A parent
may change its marketing percentage at any time for future accrual. There are no
family shares, permanent capacity rights, collateral tranches, guaranteed yields,
or mandatory marketing commitments.

## 2. Tree and admission

### 2.1 Geometry

- There is one logical, non-earning protocol root. It holds no eligible assets,
  earns no commission, and operates no family marketing pool.
- Auctioned top-level members have earning depth `d = 0`. The protocol root does
  not count as a paid ancestor. An implementation may represent these members as
  parentless records under one protocol instance rather than store a root node.
- Every ordinary node chooses exactly one existing parent when it joins. Its
  depth is its parent's depth plus one.
- Parent connections are permanent. Nodes cannot move, insert an intermediary
  into an existing connection, sever their ancestry, or form cycles.
- Maximum earning depth is `H`. A node at depth `H` cannot accept children, but
  can hold assets, earn core credits, and escrow credits with its own parent.
- Emptying a node's balances does not delete its identity, ancestry, or children.
- One owner may control multiple nodes. No economic rule depends on detecting
  whether two owners are the same person.

Let `q` be the fixed commission fraction paid to each earning ancestor. The
intended referral deployment chooses:

```text
H is a finite positive integer
q > 0
H * q < 1
```

The last condition leaves a positive own-asset share at every permitted depth.
`H` and `q` are fixed network terms once operation begins; changing them is not an
ordinary parent action covered by this specification.

The reusable kernel leaves these values to the host. Its depth representation
is `u8` (`0..=255`), with no special maximum of seven. It also permits `H = 0`
(only earning seats) and `q = 0` (no ancestor commissions); the conservation
constraint remains `H*q < 1`. These degenerate configurations do not provide
the recruitment incentives of the positive-commission deployment above.

### 2.2 Scarce top-level seats

The protocol's auction module controls top-level admission. Its configured supply
or issuance schedule must enforce scarcity; an unrestricted ordinary join action
must not create additional top-level seats.

An accepted auction result identifies the winner, the seat allocation, and the
final credit price. Activation must verify that authorization, consume it exactly
once, burn the required already-issued credits, and create a depth-zero node.
The reference operation makes payment and activation atomic. No payment goes to
the protocol root, another member, or a marketing pool.

Bid collection, clearing price, timing, and refunds belong to the auction module.
Do not treat losing or refundable bid escrow as an admission payment to burn.
Any additional auction fee or forfeiture requires its own explicit auction rule.

**Bootstrap prerequisite:** the first credit-paid auction requires an actual
pre-existing credit supply. Deployment must document its genesis or bootstrap
credit inventory. This referral model does not mint credits to pay its own auction
and does not silently grant a free first seat. Buying credits elsewhere does not
solve this prerequisite unless such credits already exist.

### 2.3 Ordinary child admission

Each node advertises two nonnegative credit amounts, in the kernel's `Offer`:

```text
current_fee: u64
increment: u64
```

A successful ordinary join:

1. Authenticates the existing parent and its immutable cached depth, checks
   available depth and unused new-node identity, and verifies the payer's
   authorization and maximum accepted fee. Joining does not load the ancestry;
   actions that operate on a path validate that complete path when supplied.
2. Transfers the current fee in already-issued credits to the parent's owner.
3. Creates the permanent child connection with zero asset balances and zero
   family credit escrow.
4. Increases the parent's quoted fee by its increment, saturating at the maximum
   representable fee amount, matching the existing `Offer` convention.

These effects are atomic. A failed join transfers nothing, creates nothing, and
does not increment the price. The parent may replace its fee and increment at any
time for subsequent joins. Zero fee and zero increment are valid and the simplest
default offer; allowing them does not guarantee that a free offer is available.

The fee is a transfer, not a burn, an emission, or the child's marketing stake.
There is no automatic sharing of admission fees. The increment is a pricing
convenience, not a security requirement or a measurement of genuine demand.

Only credits are used for these commercial fees. SOL transaction and account
funding costs are separate host responsibilities.

## 3. Asset lanes and core allocation

### 3.1 Notation

Rate formulas below describe ideal arithmetic at time `t`; time arguments may
be omitted for readability. Interval entitlements integrate these rates using
the balances and terms in effect throughout that interval.

| Symbol     | Meaning                                                         |
| ---------- | --------------------------------------------------------------- |
| `E(t)`     | Global credit emission rate, decreasing continuously over time  |
| `e_l(t)`   | Credit emission assigned to lane `l` by the external allocator  |
| `p_i,l(t)` | Node `i`'s own eligible asset balance in lane `l`               |
| `D_i,l(t)` | Eligible balance of all strict descendants of `i` in that lane  |
| `P_l(t)`   | Sum of all nodes' own eligible balances in that lane            |
| `d_i`      | Number of earning ancestors above node `i`                      |
| `s_i(t)`   | Issued credits escrowed by `i` with its immediate-parent family |
| `S_i(t)`   | Sum of `s_c` over the immediate children of `i`                 |
| `mu_i(t)`  | Parent `i`'s discretionary marketing fraction, in `[0, 1]`      |

`D` excludes the node's own balance. `S` includes only immediate children's credit
escrow, not their assets, descendants' escrow, or uncollected credit entitlements.

### 3.2 Allocator interface

The emission module supplies a nonnegative, continuous rate `E(t)` that decreases
while emission is active. The scheduled budget from `t_a` to `t_b` is
`integral(t_a..t_b, E(t) dt)`. Cumulative scheduled emission continues to grow
while `E(t) > 0`; a declining rate does not mean previously emitted credits
disappear. Actual issued supply also reflects collection and burns.

The allocator supplies authenticated, chronological lane allocations. A new
weight takes effect prospectively after the previous allocation interval is
settled. A current lane weight multiplied by an entire past interval is not an
acceptable substitute for that history.

In ordinary active operation, the allocator must route the full scheduled budget:
`sum_l e_l(t) = E(t)`. With one active lane and no gauge competition, that lane gets
`E(t)`. The allocator cannot silently retain an arbitrary portion or introduce a
second emission-rate policy.

An unavailable allocation, an empty destination lane, or numerical routing
residuals must be reported explicitly as unallocated, unissued budget. At the
routing stage, lane allocations plus unallocated routing amounts reconcile to
the scheduled budget. Within each lane, earned allocations and that lane's
unallocated amounts reconcile to the amount it received. Do not count an empty
lane's allocation again as an additional routing exception. These exceptions are
not extra rewards for later participants. This module does not compare raw BTC,
ETH, SOL, or other asset quantities across lanes.

Each lane has its own authoritative eligible balances and total `P_l`. Assets
outside the referral network do not acquire referral emission claims. The host
must prevent the same underlying eligible position from being duplicated by
registering additional accounts.

### 3.3 Core rates

For a lane with `P_l > 0`, define:

```text
own_income_i,l = e_l / P_l * (1 - d_i * q) * p_i,l
override_i,l   = e_l / P_l * q * D_i,l
core_income_i  = sum_l(own_income_i,l + override_i,l)
```

Every asset unit retains `1 - d*q` of its gross attribution at its holder and
funds one `q` commission for each of its `d` earning ancestors. Consequently the
core allocations sum to each eligible lane's assigned emission. The total
descendant balance includes children, grandchildren, and every further permitted
generation; a commission does not decay with distance.

The conservation identity is `sum_i D_i,l = sum_i d_i*p_i,l`: every source asset
appears once in each ancestor's strict-descendant total. Therefore the core
numerators sum to `P_l - q*sum_i d_i*p_i,l + q*sum_i D_i,l = P_l`.
Equivalently, each source distributes `(1-d*q) + d*q = 1` of its attribution.

A node can earn commissions from real descendant assets without holding assets
itself. A node with neither own assets nor descendant assets generates no core
income. Credit escrow is not another asset lane and creates no core entitlement.

If an allocator assigns emission to a lane with `P_l = 0`, record it as
unallocated. Do not give historical empty-lane emission to a later depositor or
silently redistribute it among other lanes. The allocator may separately choose
not to allocate to empty lanes. With no eligible allocation, no reward is earned.

### 3.4 What recruitment improves

Compare allocations at the same instant in one lane, holding its emission rate
`e`, other balances, and allocation policy fixed. Let a node's core numerator
be `K=(1-d*q)*p+q*D`, with global eligible balance `P>0`. Recruiting `x` new
asset units into its downline changes its core income by:

```text
before = e*K/P
after  = e*(K+q*x)/(P+x)
change = e*x*(q-K/P)/(P+x)
```

Absolute core income increases only when its prior budget share `K/P` is less
than `q`. The same incoming capital joining outside its subtree would instead
leave the numerator at `K`; recruiting wins `e*q*x/(P+x)` relative to that
alternative. This rewards competing for capital entering the network without
promising that a large existing holder benefits from global dilution. Marketing
and incoming gifts further affect net income; declining emission affects income
across time. Neither is held fixed implicitly in a future-return quote. This
comparison is denominated in credits; any effect of growth on their clearing
value belongs to the wider platform economics.

## 4. Discretionary credit-weighted mutualization

Each node chooses `mu_i`, independently of its join fee. It can change `mu_i`
prospectively without the children's permission. A node with `mu_i = 0` retains
all of its referral commissions.

The source of its marketing budget is its **gross referral override income**
across all lanes:

```text
O_i = sum_l(override_i,l)

M_i = mu_i * O_i    if S_i > 0
M_i = 0            if S_i = 0

gift_i,c = M_i * s_c / S_i    for an immediate child c, when S_i > 0

retained_income_i = core_income_i - M_i + gift_parent(i),i
```

A top-level seat has no incoming family gift. When `S_i = 0`, the parent retains
the otherwise undistributable budget. Its own-asset income, admission fees,
auction payments, escrow principal, and incoming family gifts are not marketing
sources. Incoming gifts are terminal: they are kept by the recipient rather than
shared again or charged another upline commission.

Every marketing payment is subtracted from one node and added to another. Thus
mutualization reallocates the existing emission budget rather than creating a
second issuance stream. The parent must retain only its net entitlement; it
cannot collect gross commissions while children also hold claims against them.

The child need not hold BTC or another asset to receive a marketing gift. Its
weight is the credits it has actually escrowed. A nephew's asset participation
can increase the grandparent's pool without increasing the uncle's competing
escrow denominator. A new sibling who only escrows credits instead dilutes the
existing siblings' share of the same pool.

This is intentional competition for a discretionary subsidy. Credit escrow alone
does not increase the parent's asset-based commissions. The parent may fund the
offer to obtain admission fees, later deposits, recruitment, or other business
benefits. No such return is guaranteed by the protocol.

## 5. Credit custody and supply

### 5.1 Family escrow

- A non-top-level member may deposit issued credits into escrow associated with
  its immutable immediate-parent relationship.
- The credits remain the member's principal. They are not paid to the parent,
  burned, or added to the parent's marketing budget.
- The same credits cannot back multiple family stakes simultaneously. One
  family's common escrow may legitimately weight all asset lanes feeding that
  family's pool; those lanes are not separate uses of the principal.
- Partial and complete withdrawals are allowed without a mandatory cooldown,
  forfeiture, or lock period. Settle earlier income before releasing principal.
- Top-level seats cannot escrow into the non-earning protocol root. Their owners
  may participate through other ordinary nodes under the normal rules.
- Uncollected earnings do not count as escrow. A convenience `collect-and-stake`
  operation must first realize the claim once and then transfer the issued
  credits into custody.
- Marketing gifts do not automatically compound. A holder may explicitly collect
  and restake them, with the new stake affecting only subsequent accrual.

The reference integration uses exclusive credit custody: family escrow, gauge
pledges, clearing commitments, and auction bid escrow cannot simultaneously spend
or reserve the same principal. Dual-use governance power over family escrow is
not specified here; adding it later requires an explicit integration rule. It
must never duplicate family reward weight or leave spent credits active in escrow.

### 5.2 Distinct financial operations

| Operation                        | Credit movement                             | Mints? | Burns? |
| -------------------------------- | ------------------------------------------- | ------ | ------ |
| Ordinary join                    | Payer to parent's owner                     | No     | No     |
| Winning top-seat payment         | Winner's issued credits destroyed           | No     | Yes    |
| Stake family credits             | Member's liquid credits to escrow           | No     | No     |
| Unstake family credits           | Escrow to member                            | No     | No     |
| Accrue core or marketing rewards | Record net entitlement                      | No     | No     |
| Collect earned credits           | Debit entitlement and issue the same amount | Yes    | No     |

For example, paying a 20-credit join fee and escrowing 80 credits requires 100
available issued credits. The child's marketing weight is 80, not 100.

Credit supply must reconcile as:

```text
issued supply = documented bootstrap supply
              + cumulative collected emission claims
              - cumulative burns
```

Custody transfers do not change supply. Burning an auction payment does not reset
the emission schedule or authorize replacement emission. To pay a fee or burn
credits currently in family escrow, first execute a correctly settled withdrawal.

## 6. Logical state

These are required logical quantities, not a prescribed account layout.

| Record      | Required state                                                                                                                                                                                                         |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Protocol    | Fixed `H`, `q`, emission clock/budget, lane registry, root-auction authorization and consumed-seat records, unallocated accounting                                                                                     |
| Lane        | Asset identity and eligibility interface, `P_l`, allocation checkpoint, cumulative reward-per-asset index `I_l`, rounding accounting                                                                                   |
| Member      | Identity and authority, immutable parent/depth, join fee/increment, `mu_i`, own escrow `s_i`, total child escrow `S_i`, family gift index `F_i`, parent gift checkpoint `f_i_seen`, uncollected net credit entitlement |
| Member/lane | Own balance `p_i,l`, strict descendant balance `D_i,l`, lane index checkpoint `I_i,l_seen`, required fractional accounting                                                                                             |
| Custody     | Issued-credit escrow backing, payer/recipient identities, burn/transfer/collection authorization and receipts                                                                                                          |

Caches must satisfy:

```text
P_l   = sum_i p_i,l
D_i,l = sum_(c immediate child of i)(p_c,l + D_c,l)
S_i   = sum_(c immediate child of i) s_c
```

Track which member/lane records can contain unsettled income. A common-family
checkpoint must use a complete authenticated set, not a caller-selected subset
of profitable lanes. Historical records must not disappear merely because their
current asset balance is zero.

New members start with zero asset balances, `s_i = S_i = 0`, `F_i = 0`,
`mu_i = 0`, and zero accrued entitlement. An ordinary child's incoming checkpoint
starts at its parent's stored `F`; a seat's unused incoming checkpoint is zero.
The first credit stake still requires the all-source barrier before activation.

Missing member/lane records represent zero contribution before their first
activation. Checkpoint the current lane index before applying a first nonzero
own or descendant balance, including first activation of an ancestor's record.
A child can receive marketing from a lane in which it has no asset record.

## 7. Accrual and the multi-lane settlement barrier

### 7.1 Reference arithmetic

The cumulative reward-per-asset index for lane `l` is:

```text
I_l(t) = integral(e_l(t) / P_l(t) dt), over intervals with P_l(t) > 0
```

Before changing a lane's total balance or allocation terms, advance its
accounting by integrating the actual scheduled lane rate using the old balance
and allocation terms. Continuous decay is part of that history; multiplying the
current rate by the entire elapsed duration is not equivalent. A member's
balances are constant between its own relevant checkpoints because every
affected asset mutation settles that member's ancestry first.

For a member/lane settlement with `delta_I = I_l - I_i,l_seen`:

```text
own      = (1 - d_i*q) * p_i,l * delta_I
override = q * D_i,l * delta_I

if S_i > 0:
    marketing = mu_i * override
    F_i += marketing / S_i
else:
    marketing = 0

kept_i += own + override - marketing
I_i,l_seen = I_l
```

`F_i` is cumulative credits allocated per escrowed credit. It aggregates
appropriately checkpointed contributions from every lane. It is not an index of
raw BTC or another asset.

To settle a child's incoming family entitlement:

```text
kept_c += s_c * (F_parent - f_c_seen)
f_c_seen = F_parent
```

These incoming credits do not update `F_c`. A child whose escrow is zero still
advances its incoming checkpoint when settled.

### 7.2 Asset changes

An asset balance change in lane `l` must:

1. Advance that lane's allocation/index to the event time using its old total.
2. Settle the affected ancestor path, from the earning seat toward the member,
   using the old lane balances and terms. Incoming gift checkpoints may be
   materialized as those parent indices advance.
3. Apply the authenticated asset change to the member's own balance, the lane's
   total, and every ancestor's strict descendant balance.

No credit escrow changes implicitly with an asset trade. Changing BTC does not
require repricing a membership agreement or creating a credit-stake contract.

### 7.3 Changes shared across lanes

**Before changing `mu_i`, or before changing any child's escrow and hence `S_i`,
bring every contributing lane of parent `i` to the same effective time under the
old `mu_i` and `S_i`.** Then update the family state.

For an escrow mutation, also settle the affected child's incoming gift under its
old `s_c` before changing that value. Other siblings need not be eagerly loaded:
their unchanged escrow balances can multiply later changes in the family index.

The reference implementation uses a complete all-source settlement barrier. It
does not need to settle every unrelated family. Since incoming gifts never enter
the outgoing marketing source, computing a parent's own gross overrides does not
require recursively materializing all of its ancestors' marketing income.

Without this barrier, a late credit depositor can capture historical earnings
from a lane that was not settled when it joined. The same defect applies to a
withdrawal or a marketing-rate change. Merely recording one aggregate family
index does not solve this problem.

Example acceptance vector:

```text
t0:   A stakes 100 credits; B stakes 0.
t10:  BTC and ETH have each accrued 100 marketing credits, not yet collected.
      Settle BOTH sources under the old stake. A owns all 200.
      B now stakes 100 credits.
t20:  Each source accrues another 50. A and B each receive 50 of this new 100.

Correct cumulative family earnings: A = 250, B = 50.
```

Settling BTC before B's deposit but assigning stale ETH income using the new
weights incorrectly gives B 50 earlier credits.

### 7.4 Execution cost and optimization boundary

Single-lane asset updates require work proportional to affected ancestry. A
credit-escrow or marketing-rate change additionally requires processing all
contributing lane sources of the affected family in the reference model. It does
not require scanning all siblings.

Do not claim lane-independent `O(depth)` cost for the complete multi-asset system.
The implementation must establish that the authenticated source set fits its
execution limits, or implement equivalent historical/versioned accounting that
preserves the reference chronology. A multi-transaction optimization must specify
its snapshot, concurrency, and commit rules; incomplete checkpoints must not
activate new stake against old income.

No arbitrary numerical lane cap, epoch duration, or economic lock period is
introduced by this document. Selecting a validated execution bound or proving an
equivalent optimized barrier is an implementation-readiness requirement.

## 8. Action contracts

The host authenticates authority, canonical records, eligible balances, and the
emission snapshot. Kernel actions validate the supplied relationships, amounts,
arithmetic, and chronological checkpoints. The host commits the returned state
and custody effects atomically; an error returns no receipt.

| Action                          | Required behavior                                                                                                                                |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| Activate auctioned seat         | Verify unique authorized allocation, burn final credit price, create a depth-zero member; consume the authorization once                         |
| Join under parent               | Authenticate parent/cached depth, respect payer's maximum fee, transfer fee to parent, create empty child, increment quoted fee once             |
| Set join offer                  | Owner changes fee and increment for future admissions; no change to existing members or accrued earnings                                         |
| Deposit/withdraw an asset       | Settle old lane/path state; authenticate custody or eligible-position change; update own balance, lane total, and ancestor totals                |
| Set marketing percentage        | Owner performs the all-source barrier; installs `mu` immediately for subsequent accrual across every lane                                        |
| Stake credits                   | Authenticate liquid issued principal; checkpoint parent sources and old child entitlement; transfer to escrow; increase child `s` and parent `S` |
| Unstake credits                 | Check available escrow; checkpoint old sources and entitlement; decrease `s`/`S`; return the same principal, without confiscating earned rewards |
| Settle lane/member              | Materialize only valid historical entitlement; permissionless settlement must not change economic ownership through cadence                      |
| Collect credits                 | Debit already accrued net entitlement and mint the same whole credit amount once; preserve the member's fractional remainder                     |
| External lane allocation change | Allocator settles old routing history before installing new allocations; no retroactive reweighting                                              |

Collection need not materialize every lazy source automatically. An operation
advertised as collecting everything through time `t` must settle the member's
own core sources and its immediate parent's gross-override sources, then advance the
member's incoming family checkpoint before collection. Iterating only the
claimant's own asset lanes is insufficient: a credit staker with no asset records
can still receive gifts from every lane of its parent. Repeated collection without
new entitlement must mint zero. An atomic collect-and-stake convenience must
preserve the same ordering and supply accounting as the separate operations.

## 9. Finite precision and liabilities

The formulas are the exact reference model. An implementation must specify its
numeric representation, rounding, and remainder ownership rather than assume
that independently flooring every operation is sufficient.

- Use checked arithmetic and prove bounds for balances, aggregate balances,
  credit escrow, rates, indices, and cumulative lifetime entitlements.
- Use the same represented `q` for own-asset deductions and ancestor commissions,
  and validate the depth bound against that effective value. Conversion or
  rounding must not make the deepest retained fraction nonpositive.
- Preserve fractional earned credits internally. Whole-token collection is not
  a reason to round each accrual to whole tokens.
- Carry or otherwise account for fractional accrual so repeated small settlements
  cannot systematically redirect material income to the parent or erase a
  child's earnings. Compare frequent settlement with one late settlement.
- Shared, not-yet-attributed remainders cannot be assigned to later arrivals.
  Establish their ownership or explicitly record them as unallocated when the
  relevant eligibility composition changes. This applies even if a replacement
  leaves the total denominator unchanged.
- An individual's already earned fractional claim stays with that individual.
  A marketing-rate change or complete withdrawal does not confiscate it.
- Track marketed-but-not-yet-claimed liabilities separately from the parent's
  spendable entitlement. Never issue both claims against the same credits.
- Record routing/empty-lane losses and rounding residuals explicitly. They must
  reconcile against scheduled emission rather than disappear from accounting.

The integer implementation must demonstrate non-overissuance and a documented
error bound under adversarial settlement and stake-change schedules. Rounding
drift proportional to an attacker-controlled number of calls, with no useful
bound, is not sufficient. Numerical validation is a release requirement, not a
claim established by the ideal formulas.

## 10. Worked allocation across two assets

Use `q = 10%`, depth at least two, and a 300-credit allocation over an interval:

```text
Protocol root: no income
└ Alice: auctioned seat, d=0; mu=50%
  └ Bob: d=1; mu=25%; 100 credits escrowed with Alice
    └ Carol: d=2; 200 credits escrowed with Bob
```

| Lane | Assigned credits | Alice's assets | Bob's assets | Carol's assets |
| ---- | ---------------: | -------------: | -----------: | -------------: |
| BTC  |              200 |            100 |          100 |              0 |
| ETH  |              100 |              0 |            0 |            100 |

Asset units are compared only within their lane. The 300 credits already in
family escrow are principal from earlier issuance; they do not enter either
asset denominator.

| Credit flow              |   Alice |       Bob |    Carol |
| ------------------------ | ------: | --------: | -------: |
| Own-asset income         |     100 |        90 |       80 |
| Gross referral overrides |      20 |        10 |        0 |
| Marketing paid           |     -10 |      -2.5 |        0 |
| Marketing received       |       0 |        10 |      2.5 |
| **Net new entitlement**  | **110** | **107.5** | **82.5** |

Net entitlement sums to 300. Bob receives part of Alice's ETH-funded override
despite having no ETH himself. Escrow principal remains untouched.

For a sibling split, a parent with a 30-credit marketing pool and children
escrowing 100 and 200 credits pays 10 and 20. Splitting the 100 into two owned
sibling accounts holding 40 and 60 pays 4 and 6: the same aggregate 10.

## 11. Economic properties and limits

The intended guarantees are specific:

- **Conservation:** core allocation divides assigned emission; marketing moves
  existing entitlements; fees and escrow do not create emission.
- **Core own-capital neutrality:** splitting fixed assets within a connected
  owned region, under unchanged outside ancestry, does not increase that owner's
  combined core entitlement.
- **Same-family escrow neutrality:** splitting the same credits among aliases
  under one parent does not increase their combined marketing share.
- **No empty-account bonus:** an account with zero credit escrow receives no
  marketing payment merely for existing. An account can still earn core
  commissions if it has real asset-funded descendants.
- **Prospective offers:** parents can change future marketing and entry prices;
  historical accrual remains governed by its historical state.

These properties concern specific capital splits and historical accounting.
They do not establish distinct human identities or universal resistance to
coalitions. In particular, they do not prohibit the following strategies.

**Owned intermediaries before an outside recruit.** With `q=10%`, `H=7`, and
zero marketing, a purchased seat's direct outside child retains 90% of its
source attribution and pays the seat 10%. If the same seat owner first creates
six owned intermediaries and admits that outside child at depth 7, the owner
collects seven commissions totaling 70%; the child retains 30%. The internal
joins can be free. Existing connections were never changed. This is a profitable
way to capture more of an outside source's attribution if the outsider accepts
that position. Available shallow offers can discipline it, but neither a scarce
seat auction nor immutable edges guarantees that those offers exist.

**Pooling above an intermediary.** For `A -> B -> C`, where A is a depth-zero
seat, C supplies all assets, `q=10%`, and marketing is zero, 100 gross credits
split as `(10,10,80)`. If A and C instead place the same assets in A's eligible
position and redistribute collected credits, they can split `(11,0,89)` and
both improve. C's old node remains attached to B but empty. This strategy
requires a usable custody/redistribution arrangement; it does not duplicate an
asset claim. If such aggregation is inexpensive and trusted, top-seat pools can
undercut multilevel positions and weaken intermediaries' future commissions.
The tree does not guarantee the continued economic use of its permanent edges.

**Discretionary subsidy recapture.** Parents may escrow credits through children
they control. With outsider escrow `S` and owned child escrow `k`, outsiders
collect `mu*O*S/(S+k)` while the parent's coalition retains
`O*(1-mu*S/(S+k))` of its gross overrides. The full escrow denominator is public,
so a newcomer can still quote its own current payout without knowing ownership.
A high `mu`, many child addresses, or many advertised links does not establish
independent competition. Fees paid to oneself remain internal transfers.

**Commercial exit and commitment.** A member may join another available parent
through a new node and move withdrawable assets and credits. The original
node's downline does not move; admission payments and past recruiting effort
are not refunded. A parent may reduce future `mu` after an admission payment.
Withdrawable principal limits custody exposure, but the current marketing rate
is not a contract for future subsidy. Reputation and future business may
support generous offers; the protocol does not prove that they will.

Credit-only stakers can receive the subsidy without depositing an eligible
asset or recruiting anyone. This is intentional. Credit escrow alone adds no
parent commission, so marketing requires a commercial rationale such as fees,
subsequent deposits, recruitment, or another external benefit. Positive sharing
is not automatically profitable. Scarce root seats constrain their quantity,
not their beneficial ownership; the design has no anti-monopoly guarantee.

With one scheduled emission budget, family growth does not guarantee every
member's income increases. Section 3.4 gives the core marginal condition.
Declining emission, gauge allocation, competing escrow, and changing offers all
affect returns. Creditless newcomers may use an available free link, earn from
assets, and later escrow credits; until then their immediate marketing weight
is zero. A free offer is permitted, not guaranteed. Borrowed capital committed
for real elapsed time remains eligible; the protocol does not identify personal,
unleveraged wealth.

The defensible market claim is therefore precise: positions and current subsidy
offers are economically distinguishable, and simple account multiplication does
not expand a fixed capital position's entitlement under the stated neutrality
conditions. Stable competition, independent recruitment, and profitable campaigns
are economic outcomes to evaluate, not consequences of conservation alone.

## 12. Acceptance tests

Use an exact rational event-driven reference model before validating the chosen
fixed-point implementation. Cover at least:

1. **Core conservation:** generated trees and multiple nonempty lanes sum to the
   assigned emission for arbitrary valid depths and balances.
2. **Marketing conservation:** arbitrary `mu`, credit escrows, and empty families
   preserve the same total; no reward becomes negative.
3. **Depth:** seats have depth zero; the protocol root earns nothing; bottom
   members cannot recruit but retain positive own-asset earnings; reject `H*q >= 1`.
4. **Owned asset splits:** compare core payouts before/after same-capital changes
   within connected owned paths, without moving existing outside connections.
5. **Escrow splits:** compare one stake with multiple same-parent aliases having
   the same combined issued-credit principal.
6. **Late staking:** a newcomer receives none of a previously accrued pool;
   stake and unstake at the same timestamp earn no intervening reward.
7. **Cross-lane boundary:** reproduce the BTC/ETH vector in section 7.3. Repeat
   with withdrawal and with a `mu` change while another lane remains lazy.
8. **No eligible escrow:** the parent retains income when `S=0`; the first staker
   receives no backpay; the last unstaker preserves its earned entitlement.
   A zero-asset staker with no own lane records can collect multi-lane gifts.
9. **Terminal gifts:** an incoming marketing gift is never shared again, even
   when the recipient's own `mu=100%`.
10. **Source separation:** own-asset yield, fees, escrow deposits, and incoming
    gifts do not increase the gross-override marketing source.
11. **Fees:** zero/zero offers, successful increments, saturation, owner repricing,
    payer price limits, insufficient funds, duplicate identities, and rollback.
12. **Root auctions:** invalid/replayed allocations fail; authorized capacity is
    respected; final payment is burned rather than transferred; activation and
    payment cannot leave a partially applied result.
13. **Custody and supply:** escrow principal is withdrawable once, cannot be
    pledged or spent twice, and is not counted as an emission claim; collections
    mint once; auction burns reduce supply without replenishing emission.
14. **Asset churn:** frequent buys/sells affect only subsequent allocation;
    lane totals, ancestor totals, and all reward checkpoints remain consistent.
15. **Precision:** repeated tiny settlements, partial escrow withdrawals,
    zero-duration calls, denominator changes, constant-denominator owner swaps,
    maximum balances, long lifetimes, and overflow rejection. Compare against
    exact arithmetic and document residual ownership/error bounds.
16. **Empty lanes:** no allocation is divided by zero or awarded retroactively;
    unallocated routing and lane amounts reconcile to the global budget.
17. **Partial settlement order:** legitimate different lane settlement orders
    reproduce the reference ownership; omitting a lane at a family-state barrier
    is rejected rather than accepted with altered payouts.
18. **Declining emission:** allocations reconcile to integrated scheduled emission
    under a continuously decreasing rate. Changing checkpoint frequency does not
    change the interval budget or apply today's rate retroactively.

## 13. Kernel implementation

The migrated `mayf-referral` kernel implements the allocation rules through
public actions; the host applies their receipts atomically.

Tree height and commission are supplied as network terms. The current `u16`
lane mask has capacity for 16 permanent asset lanes; this storage choice does
not fix tree height. An `H = 7` deployment has eight earning members per path,
but is only one configuration. Hosts must validate the execution costs of their
chosen height and authenticate the independently configured emission schedule.

| Surface                                                            | Implemented behavior                                                                |
| ------------------------------------------------------------------ | ----------------------------------------------------------------------------------- |
| [`program/join_at_top_level.rs`](src/program/join_at_top_level.rs) | Consume an authenticated auction award and return its credit burn                   |
| [`program/join_under_parent.rs`](src/program/join_under_parent.rs) | Permanent parent, cached depth, payer fee limit, and saturating offer increment     |
| [`program/set_balance.rs`](src/program/set_balance.rs)             | Reconcile one asset balance after settling its ancestor path                        |
| [`program/set_sharing.rs`](src/program/set_sharing.rs)             | Complete source barrier before prospective marketing changes                        |
| [`program/set_credit_stake.rs`](src/program/set_credit_stake.rs)   | Complete parent barrier, old gift settlement, and explicit escrow custody change    |
| [`program/settle_family.rs`](src/program/settle_family.rs)         | Current incoming gifts, including for zero-asset children                           |
| [`program/collect_credits.rs`](src/program/collect_credits.rs)     | Mint accrued net entitlement and retain fractions                                   |
| [`emission.rs`](src/emission.rs)                                   | Authenticated cumulative emission input, separate from token supply                 |
| [`lib.rs`](src/lib.rs)                                             | Numeric representation, residual ownership, bounded working sets, and host contract |

Actual asset changes close upstream gauge residuals. Actual family changes also
commit and close every parent source lane's residuals, then close shared family
carries. These amounts become explicitly unallocated. This prevents old upstream
dust arriving under new eligibility, at the cost of shared writes and bounded
rounding loss. Private own-income and retained-override carries survive changes.
The code documents the bounds in credit base units.

The external host still supplies authenticated accounts, the emission schedule,
auction allocation, asset eligibility, exclusive custody, and atomic transfers,
burns, and minting.

## 14. Deployment and implementation readiness

Before deployment, specify and validate:

- Numerical `H`, `q`, and the global emitter's initial rate and continuous decay
  schedule.
- The credit mint, token precision, custody authorities, and documented bootstrap
  credit supply.
- The scarce seat supply/issuance authorization and the external auction format.
- The lane registry, asset eligibility adapters, and external allocation history.
- A complete multi-lane checkpoint implementation within execution limits.
- Numeric bounds, remainder ownership, and the acceptance tests above.

These are implementation and deployment prerequisites. They do not introduce
additional parent pricing restrictions, guaranteed family rights, a new gauge
mechanism, or a particular decay curve into the tree algorithm.

The far-future check of total issued credit supply against `u64::MAX`, measured
in token base units, is deferred to later implementation work. This document
does not choose an overflow policy or infer a safe lifetime supply bound from
decay alone. Ordinary checked arithmetic and reward-conservation requirements
still apply.
