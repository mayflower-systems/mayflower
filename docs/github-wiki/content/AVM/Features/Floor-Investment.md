<!--
title: Floor Investment
summary:
  Raising the floor with existing reserve surplus while preserving main pricing.
tags:
  - avm-features
  - floor
-->

Floor investment names a surplus-funded [floor raise](AVM-Floor-Raising). It
permits required backing to increase while preserving the market's live price
and entire main schedule.

Donation and floor raising do not themselves create a funder position, repayment
balance, or reward allocation. The name describes a reserve-accounting
operation.

## Available surplus

Let $H$ be held reserve, $D$ outstanding collateralized advances, and $B$ the
required backing from the [AVM specification](AVM-Specification). Then

$$
S=H+D-B\ge0
$$

is backing surplus. In particular,

$$
B=F(T-P+A)+\int_A^P y(x)\,dx,
$$

where $F$ is the floor, $T$ circulation, $P$ progress, and $A$ the ramp start.
This includes floor issuance and redemption when $T\ne P$.

A donation increases $H$ without increasing $B$. It creates surplus but does not
itself change the floor. A subsequent authorized, valid raise is required. A
reserve advance merely exchanges held reserve for an outstanding advance; it
does not increase $S$.

## Accepted proposal

The proposal increases $F$ and supplies a new ramp end $E'$ and width $W'$, with
$A'=E'-W'$. It preserves the characteristic, ramp multiplier, main offset,
progress, and circulation. It must satisfy

$$
A'\ge A,\qquad E'\ge E,\qquad 0\le A'\le E'\le P.
$$

The floor join must be continuous in the exact model, or have only an allowed
upward gap in bounded arithmetic. The ramp may narrow or collapse; its end may
stay fixed or advance. The live price stays on unchanged main.

For a nonnegative reserve budget $G$, numerical acceptance requires the entire
backing-change enclosure to satisfy

$$
0\le B'-B\le G,
$$

and the upper bound for $B'$ must be covered by $H+D$. The budget is a maximum
permitted increase, not a transfer of funds. No reserve balance changes during
the raise itself.

## Limits

Surplus alone does not make an arbitrary floor reachable. With main preserved
and $E'\le P$, the floor cannot exceed the live price in a valid continuous
proposal. For example, if the live price already equals the floor, preserving
that live price prevents a positive increase even when surplus exists.

Donated capital strengthens backing immediately. How much becomes floor backing
depends on the accepted geometry and the outstanding circulation. There is no
one-unit-donation-to-one-unit-floor conversion.

## Relationship to ordinary floor raising

| Property                             | Ordinary floor raise                     | Floor investment                                |
| ------------------------------------ | ---------------------------------------- | ----------------------------------------------- |
| Main schedule and live price         | Preserved                                | Preserved                                       |
| Floor                                | Increases                                | Increases                                       |
| Progress and circulation             | Preserved                                | Preserved                                       |
| Required backing in exact arithmetic | Unchanged                                | May increase                                    |
| Numerical backing-change allowance   | $[-\epsilon_B,0]$                        | $[0,G]$                                         |
| Source of additional floor backing   | Reallocation within existing requirement | Existing surplus plus any internal reallocation |

Both use the same geometry and coverage checks.
[Linear AVM Floor Investment](Linear-AVM-Floor-Investment) gives an exact worked
example.
