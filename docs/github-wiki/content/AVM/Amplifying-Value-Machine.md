<!--
title: Amplifying Value Machine
summary:
  Entry point for the AVM — a deterministic pricing function that maps supply to
  price across three segments (floor, ramp, main), bootstraps its own liquidity
  from reserves, and guarantees a monotonically non-decreasing floor price for
  every token in circulation.
tags:
  - avm
  - pricing
-->

The Amplifying Value Machine (AVM) is a deterministic pricing function that
**is** the liquidity of a market. There is no order book, no liquidity pool, no
external market maker. The AVM maps token supply to price, collecting capital
into a reserve on every buy and paying it out on every sell. The reserve always
equals the area under the price curve — every token in circulation is fully
backed at the price the curve assigns.

The asset it issues is an **Amplified Value Asset (AVA)**. Its minimum
redemption value is denominated in the market's reserve asset, and its floor can
only stay fixed or rise. Measured from its initial floor, an unencumbered AVA's
minimum reserve claim cannot underperform the equivalent initial quantity of its
reserve asset, before fees. This does not guarantee the reserve asset's value in
another denomination or protect a holder who paid more than the floor. An AVA
used to draw a cash advance is encumbered; the holder cannot both withdraw its
floor backing and redeem it freely.

The AVM is a species of automated market maker (AMM), but unlike conventional
AMMs it does not depend on external liquidity providers. It bootstraps its own
liquidity: buyers create the reserve by purchasing tokens, and the reserve is
the counterparty to every subsequent sell. The AVM also evolves — its floor
price can rise over time, and its sensitivity adapts as the market contracts and
expands. It additionally provides credit liquidity through the
[cash advance](Cash-Advance-Overview) facility, which lets
holders draw against the floor value of their collateral.

---

## Notes

- [Specification](AVM-Specification) — General definition of the price curve:
  three segments (floor, ramp, main), the ramp constraint, solvency invariant,
  buying, selling, contraction, and the guaranteed-vs-discovery decomposition.
  Expressed in terms of general monotonic functions, not a specific
  implementation.
- [Floor Raising](AVM-Floor-Raising) — The area-preserving floor raise: why
  the ramp constraint creates an area deficit, how it funds a higher floor,
  invariants, preconditions, magnitude bounds, and the sell–raise cycle.
- [Evolution](AVM-Evolution) — Complete walkthrough of every curve mutation:
  buy, sell, sell into ramp, sell into floor, floor raise, floor investment, and
  option execution. State-change tables and the self-healing sell–raise cycle.
- [Rationale](AVM-Rationale) — Why the AVM is a distinct class of AMM: no
  liquidity providers, self-bootstrapping reserve, evolving floor, credit
  liquidity.
- See also [AVM Features](AVM-Features) for optional add-ons built on top of the core AVM
  (call options, floor investment, etc.).

---

## Key Concepts

- **Three segments** — Floor (constant at $f$), ramp (monotonically increasing,
  steep), main (monotonically increasing, gentle). The floor is the guaranteed
  minimum. The ramp is the transition zone. The main is the long-run pricing
  schedule.
- **Reserve = area under curve** — The solvency invariant. At every moment, the
  reserve holds enough capital to redeem every token at the price the curve
  assigns. No token is unbacked.
- **Floor** — The guaranteed minimum redemption price. Selling into the floor
  has zero price impact. The floor can only rise, never fall.
- **Floor raising** — Converts volatile area into guaranteed area without
  changing the reserve or the spot price. Works because the ramp sits below the
  main function in the transition region, creating an area deficit that funds
  the higher floor.
- **Contraction** — When supply is sold below the ramp–main boundary, the main
  schedule shifts down. This makes future floor raises possible at lower supply
  levels — the system is self-healing.

---

## How it connects

- Market authorities configure fees and operating policy. Reserve's
  [channel weights](Emission-Channel-Weighting) direct emission among
  markets; they do not change the price curve or its parameters.
- In Reserve, eligible deposited AVA units supply emission points after
  [referral enrollment](Referral-Network). Market prices affect the cost of
  acquiring those units; the outlet's allocation and competing points determine
  their reward share.
- The [cash advance](Cash-Advance-Overview) facility draws
  liquidity from the same reserve that backs the price curve.
