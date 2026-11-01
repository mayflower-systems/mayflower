# Reward Clearing

## State and parameters

The [algorithm state types](rust/mayf-reward-clearing/src/state.rs) carry:

| State           | Essential data                                                                                                                    |
| --------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| Epoch schedule  | Immutable master start `t0` and duration `T`, shared by synchronized channels.                                                    |
| Active epoch    | Schedule/counter, immutable terms, accepted principal `A`, surviving shares `S`, tracked net reward `R`.                          |
| Lot             | Whole remaining principal `Aij`, remaining shares `Sij`, original entry weight.                                                   |
| Position        | Aggregate principal and shares, and whether its entitlement has been claimed.                                                     |
| Finalized epoch | Schedule and terms, immutable settlement totals `S*` and `R*`, remaining unclaimed shares `U` and reward `V`; no principal field. |

Tenant and channel identities, channel configuration, user authorities, staged
balances, lot IDs, and lot collections belong to the program application. A
different consumer can select its own identities and storage representation. The
core accepts the position aggregate and its newest lot; the host must select
that lot correctly and derive the aggregate from the associated position.

Parameters are:

- `T > 0`: cadence in seconds; the Solana host requires a new tenant to start
  strictly in the future.
- `0 ≤ b ≤ 2`: linear entry bonus slope.
- `k ∈ {1, 2, 3, 4}`: exit penalty exponent.
- `0 ≤ f ≤ 1`: live environment reward-funding fee applied by the host before
  core funding.

Each epoch snapshots `T`, `b`, and `k`. The channel authority may change `b` and
`k` for subsequent epoch creation; existing epochs keep their terms. Disabling a
channel prevents new epochs and does not disable existing ones. The fee is
checked on each funding transaction, rather than fixed for an epoch. The
reference choices `b = 1` and `k = 2` are examples, not implicit defaults.

Application terms express `b` in micro-basis points (`1_000_000` represents
one); the numeric projection rounds it down. Fee rates use the validated
`MicroBasisPoints` type with the same scale. Amounts transferred are whole `u64`
accounting units; the consumer determines what those units represent and selects
the numeric driver's precision and range. The implementation widens whole
amounts only for share or entitlement calculations; it never stores fractional
principal or reward credit. Funding rejects totals outside that driver's
representable range before accepting a reward pool that could not later be
claimed.

Smart constructors and snapshot conversion validate local state: representable
schedule boundaries, bounded terms, principal/share consistency, valid lot
weights, and finalized remainders no greater than their frozen totals. The
program authorization rules validate relationships and authorities at handler
entry. Global sums below hold for histories built from these transitions; a
standalone snapshot cannot prove that no other stored positions exist.

## Epoch schedule

For authenticated time `now ≥ t0`:

$$
n = \left\lfloor\frac{now-t_0}{T}\right\rfloor,\qquad
start_n=t_0+nT,\qquad end_n=start_n+T.
$$

The live interval is `[start_n, end_n)`. The Solana initializer accepts only its
current counter and checks the caller's expected channel terms. Time before
`t0`, a counter exceeding `u32`, or overflowing timestamps returns an error.
Solana uses `Clock::unix_timestamp`; negative timestamps fail its numeric
conversion.

There is no stored current clock, clock-advance action, or predecessor
finalization requirement. Missed empty windows need no accounts. Earlier epochs
may await finalization or claims while the current epoch proceeds. The program
cannot recreate an ended historical epoch.

## Reward funding

For gross whole reward amount `g` and live fee `f`, funding computes:

$$
F=\lceil gf\rceil_{\mathrm{unit}},\qquad N=g-F,\qquad R'=R+N.
$$

The program's receipt sends `F` to protocol custody and `N` to the epoch. Their
sum is exactly `g`; principal penalties never enter `R`. The caller supplies a
maximum acceptable fee rate, checked before the action succeeds. At a 100% fee,
all funding goes to the protocol. Rounding can also consume a small gross
deposit entirely at a lower nonzero fee.

The core receives only `N` and returns the increased tracked reward balance. It
has no protocol fee, fee authority, or funding-quote policy.

Funding is accepted only in the live interval. Direct transfers into an epoch
vault are surplus, not reward funding, and never change its tracked `R`.

## Deposits and entry weight

Let `t = now - start`, with `0 ≤ t < T`. The ideal entry curve is:

$$
w_b(t)=1+b\left(1-\frac{t}{T}\right).
$$

A deposit of positive whole principal `a` creates a new lot with that entry
weight. In exact arithmetic its shares are `a w_b(t)`. Earlier entry receives at
most `1+b ≤ 3` times the weight of the endpoint; `w_b(T)=1` is a curve limit,
not an allowed deposit time.

The executable rule uses downward rounding to the numeric grid. Write `floorε`
and `ceilε` for directed rounding to that grid:

$$
u=\operatorname{floor}_{\varepsilon}\left(\frac{T-t}{T}\right),\qquad
w=1+\operatorname{floor}_{\varepsilon}(bu),\qquad
s=\operatorname{floor}_{\varepsilon}(aw).
$$

The core requires the new entry weight to be no greater than the supplied head's
entry weight. The program requires `s` to meet the caller's minimum shares. It
pushes `(a, s, w)` and updates `A'=A+a`, `S'=S+s`. Existing lots retain their
own weights; there is no blended entry time.

The Solana host supports at most 32 live lots per position. IDs increase
independently of the live count. Fully removing a lot frees capacity; exhausted
ID space or a full stack rejects another deposit without disabling withdrawal or
settlement.

## Withdrawals and exit penalty

One withdrawal action consumes only the current head lot. For requested whole
principal `x`, require `0 < x ≤ Ah`, where `(Ah, Sh)` are that head's remaining
principal and shares. A request spanning lots requires repeated actions against
the newly exposed head, not one unbounded traversal.

Shares removed are:

$$
\Delta S =
\begin{cases}
S_h,&x=A_h,\\
\min\left(S_h,\operatorname{ceil}_{\varepsilon}(xS_h/A_h)\right),&x<A_h.
\end{cases}
$$

The ideal principal penalty depends on absolute epoch time, not lot age:

$$
p_k(t)=\left(\frac{t}{T}\right)^k.
$$

For the executable penalty, first compute `v = ceilε(t/T)`. The numeric trait's
`checked_pow_ceil(k)` raises `v` to `k` by exponentiation by squaring, rounding
every multiplication upward; clamp the result `p` to one. Thus grid
approximation never understates the ideal penalty. The returned whole principal
and forfeiture are:

$$
B=\left\lfloor\operatorname{floor}_{\varepsilon}(x(1-p))\right\rfloor_{\mathrm{unit}},
\qquad C=x-B.
$$

The core subtracts `x` and `ΔS` from both the lot/position and epoch aggregates.
The program checks `B` against the caller's minimum return. A fully consumed lot
is popped. Removed shares are destroyed; `C` is forfeited, not paid to the
remaining participants. Exact principal conservation is `B+C=x`.

At the start, the ideal and executable penalty are zero. At the deadline the
curve reaches one, but withdrawals are already closed. A newly created wallet or
fresh lot cannot restart the absolute penalty clock.

## Settlement and claims

After `now ≥ end`, core settlement reports all tracked principal as forfeited
and freezes:

$$
S^*=S,\qquad R^*=R,\qquad U=S^*,\qquad V=R^*.
$$

The active epoch is consumed. A finalized epoch contains no principal. Merely
reaching the deadline does not execute this transition or reduce mint supply.

The program checks that live principal and reward custody back tracked amounts.
It forfeits the entire live clearing balance, including unsolicited excess. Live
vault balances never enter the core calculation.

When `S* = 0`, the program finalization returns a terminal closure outcome
immediately: the entire live reward pot belongs to protocol custody. It does not
create an empty claimable state or carry reward into another epoch.

For `S* > 0`, a participant with surviving aggregate shares `Si = sum_j Sij`
receives:

$$
P_i=\left\lfloor\frac{R^* S_i}{S^*}\right\rfloor_{\mathrm{unit}}.
$$

The core uses a fused multiply/divide with downward rounding, then rounds to a
whole accounting unit. It does not first approximate `Si/S*` independently.
Every claim uses the same frozen numerator and denominator:

$$
U'=U-S_i,\qquad V'=V-P_i.
$$

The core returns `P_i` as earned reward. The program adds it to the
participant's staged balance with checked integer arithmetic:
`staged_i' = staged_i + P_i`.

The position is consumed once. Its exact shares are retired even when its
whole-unit payout is zero. No fractional user entitlement is persisted or
carried into another epoch. Claims have no deadline. Their backing remains in
the finalized epoch until claimed, after which it backs the user's staged
balance. Collection transfers all that user's staged reward and returns its
balance to zero.

Only `U = 0` lets the core report its terminal tracked remainder. The program
uses that condition to permit account closure. The complete live reward balance,
including tracked remainder and unsolicited excess, goes to protocol custody. No
last claimant receives another claimant's fractional remainder. An empty user
position can be closed after its copied immutable epoch end without requiring an
active or finalized epoch account to remain available.

Finalization and claims are one-shot transitions. A host must retire their
consumed state. Replaying a previously computed pure receipt is not a valid
second operation against unchanged persistence.

## Conservation and rounding

For a reachable active epoch:

$$
A=\sum_{i,j} A_{i,j},\qquad S=\sum_{i,j} S_{i,j},\qquad
A\le S\le (1+b)A.
$$

Every deposit adds its own principal and shares to these sums; every withdrawal
subtracts the same amounts from its lot and epoch. No principal or share value
is transferred into another epoch.

Reward conservation follows from the immutable claim ratio. After all claims:

$$
D=R^*-\sum_i P_i\ge0,\qquad
\sum_i P_i+D=R^*.
$$

For `m` positive-share positions, `0 ≤ D < m` whole reward base units. This
bound covers tracked rounding remainder; unsolicited surplus is additional.
Including funding fees, each gross unit is assigned to participants or the
protocol:

$$
\sum g=\sum F+\sum_i P_i+D.
$$

This identity assumes the positive-share case and all claims completed; a
zero-share epoch assigns its entire net reward to the protocol. Staging moves a
liability and its backing without changing its amount. Collecting discharges
that liability.

Principal accounting has an independent identity: total accepted deposits equal
returned principal plus withdrawal forfeitures plus principal retained until
finalization, which is then forfeited. Reward and principal can be different
assets and must not be added together as one economic quantity.

The directed rounding policy is deliberate: entry weight/shares down, penalty
and partial share removal up, returned principal and claims down to whole units,
funding fee up to whole units. There is no mutable-remainder division, implicit
dust donation to later epochs, or extra minting to absorb rounding.

For a fixed total share entitlement, splitting it across claimants cannot
increase its total payout because `sum floor(xi) ≤ floor(sum xi)`. In the ideal
lot model, withdrawing the lowest-weight lot sacrifices the fewest shares for a
given principal reduction; decreasing entry weights make LIFO that choice.
Native lots expose it without requiring a new identity for each deposit.
Whole-unit refund rounding is per withdrawal action: crossing lot boundaries can
lose more return to rounding than withdrawing the same principal from one larger
lot. Separate identities can therefore offer a tradeoff between extra principal
returned and additional shares surrendered. These are scoped arithmetic
properties, not a guarantee of identical payoffs for every wallet arrangement.
Extra identities also permit more than the Solana host's 32 lots in total;
timing, external information, revenue control, and collusion affect economic
outcomes.

For example, take an older 100-unit lot at weight 2 and a newest 1-unit lot at
weight 1.5. At a 25% penalty, withdrawing two units through LIFO requires two
one-unit actions: each returns `floor(0.75) = 0`, and together they remove 3.5
shares. Holding the lots under separate authorities allows one two-unit
withdrawal from the older lot, returning `floor(1.5) = 1` while removing 4
shares. Neither outcome dominates without valuing the lost shares.
