# Reward Clearing

The [Solana program](sol-programs/reward-clearing/README.md) uses the
`prog/`, `bin/`, `cpi/`, and `idl/` layout. Run
`./reward-clearing/sol-programs/reward-clearing/build.sh` from the repository
root to build its binary, IDL, and generated CPI crate.

## State and parameters

The [algorithm state types](rust/mayf-reward-clearing/src/models.rs) carry:

| State           | Essential data                                                                                                                    |
| --------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| Epoch schedule  | Immutable master start `t0` and duration `T`, shared by synchronized channels.                                                    |
| Channel epoch   | Channel/counter, immutable terms, accepted principal `A`, surviving shares `S`, tracked net reward `R`.                           |
| Lot             | Whole remaining principal `Aij` and remaining shares `Sij`.                                                                      |
| Position        | Channel epoch identity, ordered lots, and whether its entitlement has been claimed.                                              |
| Finalized epoch | Frozen settlement totals `S*` and `R*`, remaining unclaimed shares `U` and reward `V`, held on the existing channel epoch.          |

Library models carry generic identities and each position's ordered lot
collection. The actions select lots and maintain the channel epoch's balances.
The host chooses its identity and storage representation and handles user
authorities and token custody.

Parameters are:

- `T ≥ 3,600`: cadence in seconds; the library requires at least one hour.
- `0 ≤ b ≤ 2`: linear entry bonus slope.
- `k ∈ {1, 2, 3, 4}`: exit penalty exponent.

Each epoch snapshots `T`, `b`, and `k`. The channel authority may change `b` and
`k` for subsequent epoch creation; existing epochs keep their terms. Disabling a
channel prevents new epochs and does not disable existing ones. The reference
choices `b = 1` and `k = 2` are examples, not implicit defaults.

Every channel is created with its first channel epoch during the group's current
live epoch. The library's `create_channel` action checks the epoch's group,
current counter, and live interval, then returns both new states. The Solana host
creates the channel, first channel epoch, and their escrows atomically. Each
channel records a mandatory `last_created_epoch_counter: u32` from creation.
Later `channel_epoch_create` calls open strictly later live epochs; a channel
can skip epochs in which it does not participate.

Application terms express `b` in micro-basis points (`1_000_000` represents
one); the numeric projection rounds it down. Amounts transferred are whole `u64`
accounting units; the consumer determines what those units represent and selects
the numeric driver's precision and range. The implementation widens whole
amounts only for share or entitlement calculations; it never stores fractional
principal or reward credit. The Solana host uses `U64F64`, which can represent
every whole `u64` reward amount.

Public actions validate model relationships, lifecycle, timing, and checked
balance changes. Term conversion validates the configured curve parameters;
Anchor handles authorization and custody. Global sums below hold for histories
built from these transitions; a standalone snapshot cannot prove that no other
stored positions exist.

## Epoch schedule

`create_channel_group` validates the one-hour minimum and checks that the initial
epoch's `start + duration` fits in `u64` seconds before returning the group and
epoch. The start may be in the past, present, or future. `advance_epoch` also
checks the successor's end before advancing the stored counter.

For authenticated time `now ≥ t0`:

$$
n = \left\lfloor\frac{now-t_0}{T}\right\rfloor,\qquad
start_n=t_0+nT,\qquad end_n=start_n+T.
$$

The live interval is `[start_n, end_n)`. The group's stored counter advances
through `epoch_create`, one epoch at a time. The library's
`Epoch::validate_active(group, now)` requires the epoch to belong to that group,
match its current counter, and contain `now` in its live interval. Channel and
channel epoch creation, bidder position creation, deposits, withdrawals, and
reward funding all use this check.

Both checks matter: delayed advancement can leave an ended epoch at the current
counter, while a clock check alone does not establish group membership or the
recorded current epoch. Solana supplies authenticated time through
`Clock::unix_timestamp`; negative timestamps fail its numeric conversion.

Individual channels may skip epochs in which they do not participate. Advancing
the group does not require earlier channel epochs to be finalized. Historical
finalization and claims remain available independently of the current counter.

## Reward funding

`add_reward` accepts a net whole reward amount `N` and checks:

$$
R'=R+N.
$$

`channel_epoch_add_to_reward_pot` transfers that exact amount into the epoch's
escrow and persists the returned balance. Principal penalties never enter `R`.
The current action and Anchor instruction have no funding fee or maximum-fee
argument. A higher-level program can charge fees before passing the net amount
to this action.

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

The action computes the entry weight from the epoch terms and authenticated time,
pushes a lot containing `(a, s)`, and updates `A'=A+a`, `S'=S+s`. Existing lots
retain their own credits and shares; there is no blended entry time.

The Solana host sets `MAX_LOT_COUNT: u32 = 8` and stores each position's lots in
a Borsh `Vec`. `create_bidder_position` only creates positions for a matching,
unfinalized channel epoch in the group's current live epoch. Position accounts
start empty and grow on deposit to fit their actual lot count; the payer covers
the additional rent. A full stack rejects another deposit without disabling
withdrawal or settlement. Fully removing a lot frees capacity for another deposit.

## Withdrawals and exit penalty

One withdrawal action removes positive whole principal `x` from the newest lots
first. It can span several lots, bounded by the position's lot limit, and rejects
an amount exceeding the position's remaining credits. For each consumed lot,
let `x_h` be the credits removed and `(A_h, S_h)` its remaining credits and shares.

Shares removed are:

$$
\Delta S =
\begin{cases}
S_h,&x_h=A_h,\\
\min\left(S_h,\operatorname{ceil}_{\varepsilon}(x_hS_h/A_h)\right),&x_h<A_h.
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

The action sums the shares removed across lots and subtracts them and `x` from
the channel epoch's balances. It computes `B` and `C` once for the entire
withdrawal. Fully consumed lots are popped. Removed shares are destroyed; `C`
is forfeited, not paid to the remaining participants. Exact principal conservation
is `B+C=x`.

The current deposit and withdrawal instructions accept an amount without a
minimum-share or minimum-return bound. Such quote protections belong in a
higher-level interface; delayed execution within the live interval uses the
entry weight or exit penalty at execution time.

At the start, the ideal and executable penalty are zero. At the deadline the
curve reaches one, but withdrawals are already closed. A newly created wallet or
fresh lot cannot restart the absolute penalty clock.

## Settlement and claims

After `now ≥ end`, permissionless `channel_epoch_finalize` calls the library's
`finalize_channel_epoch` action, burns the recorded clearing credits, and freezes:

$$
S^*=S,\qquad R^*=R,\qquad U=S^*,\qquad V=R^*.
$$

The existing channel epoch stores `final_totals: Option<FinalizedTotals<N>>`;
there is no separate finalization account. `Some` records completion, including
when either total is zero. The recorded clearing-credit balance remains as
historical accounting after the burn. Merely reaching the deadline does not
finalize the epoch or reduce mint supply. Unsolicited vault transfers never
change the recorded totals.

For `S* > 0`, a participant with surviving aggregate shares `Si = sum_j Sij`
receives:

$$
P_i=\left\lfloor\frac{R^* S_i}{S^*}\right\rfloor_{\mathrm{unit}}.
$$

The core uses a fused multiply/divide with downward rounding, then rounds to a
whole accounting unit. It does not first approximate a per-share reward rate.
Every claim uses the same frozen numerator and denominator:

$$
U'=U-S_i,\qquad V'=V-P_i.
$$

`bidder_position_claim` requires the position's authority and calls the library's
`claim` action. The library checks the position's channel and epoch, sums its
surviving lot shares, and returns both updated states and `P_i`. Anchor transfers
that reward directly from the escrow to the chosen destination, persists the
channel epoch, and closes the position account, returning its lamports to the
authority. The destination may have any owner. Failed transfers roll back the
claim and closure.

Each position can claim once. The library marks it claimed, and the host closes
it. Its exact shares are retired and its lamports returned even when its
whole-unit payout is zero. Claiming requires surviving shares; an empty position
cannot close through this instruction. Creation rejects finalized or expired
epochs, so a closed position cannot be recreated to claim again. Claims have no
deadline, need no channel-group advancement, and never burn credits again. A
client may put finalization followed by its claim in the same transaction;
finalization does not automatically sweep other channel epochs.

Unclaimed rewards and rounding remainder remain in the reward escrow. No last
claimant receives another claimant's fractional remainder. If `S* = 0`, no
position can claim and the entire reward pot remains. These instructions do not
close channel epochs or sweep protocol remainder.

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
Across net funding amounts accepted by the action:

$$
\sum N=\sum_i P_i+D.
$$

This identity assumes the positive-share case and all claims completed. A
zero-share epoch retains its entire net reward as remainder. Paying a claim
reduces the reward liability and its escrow backing by the same amount.

Principal accounting has an independent identity: total accepted deposits equal
returned principal plus withdrawal forfeitures plus principal retained until
finalization, which is then forfeited. Reward and principal can be different
assets and must not be added together as one economic quantity.

The directed rounding policy is deliberate: entry weight/shares down, penalty
and partial share removal up, returned principal and claims down to whole units.
There is no mutable-remainder division, implicit dust donation to later epochs,
or extra minting to absorb rounding.

For a fixed total share entitlement, splitting it across claimants cannot
increase its total payout because `sum floor(xi) ≤ floor(sum xi)`. In the ideal
lot model, withdrawing the lowest-weight lot sacrifices the fewest shares for a
given principal reduction; decreasing entry weights make LIFO that choice.
Native lots expose it without requiring a new identity for each deposit.
Whole-unit refund rounding happens once per withdrawal action, including when
it spans lots. Splitting a withdrawal into separate actions can reduce the total
refund: at a 25% penalty, one two-unit withdrawal returns one unit, while two
one-unit withdrawals each return zero. The host's eight-lot limit is per
position; extra identities can hold additional positions. These arithmetic
properties do not guarantee identical outcomes for every wallet arrangement.
