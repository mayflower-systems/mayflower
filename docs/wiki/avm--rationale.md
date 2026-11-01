---
title: Rationale
summary:
  Why the AVM is a distinct class of automated market maker — no liquidity
  providers, self-bootstrapping reserve, evolving floor, and credit liquidity
  from the same capital pool.
tags:
  - avm
  - pricing
---

The AVM is a species of automated market maker (AMM), but it differs from
conventional AMMs in several structural ways. This note explains what makes the
AVM a distinct class and why those differences matter.

---

## What it shares with AMMs

The AVM is an AMM in the sense that it is a deterministic pricing function that
acts as the counterparty to every trade. A buyer sends capital and receives
tokens; a seller sends tokens and receives capital. The price is determined by
the current state of the machine — there is no negotiation, no order matching,
no human discretion. In this respect, the AVM resembles any on-chain AMM:
Uniswap, Curve, Balancer, and so on.

---

## Where it diverges

### No liquidity providers

Conventional AMMs depend on external liquidity providers — entities that pledge
capital to the pool, and the AMM allocates that capital according to a pricing
function. Buyers and sellers trade against the provided liquidity. The liquidity
provider is the market maker in an economic sense; the AMM merely automates the
order-filling. If liquidity providers withdraw, the AMM becomes illiquid.

The AVM has no liquidity providers. The reserve is created by buyers themselves:
every buy adds capital to the reserve, and the reserve is the counterparty to
every sell. The AVM bootstraps its own liquidity from first principles. There is
no external party whose participation is required for the market to function,
and no external party who can withdraw liquidity and leave the market stranded.

### Self-bootstrapping reserve

Because the reserve equals the area under the [[avm--specification|price curve]]
from zero to supply, the reserve is always exactly sufficient to redeem every
token at the price the curve assigns. Solvency is a mathematical identity, not a
condition that must be monitored or maintained. The reserve cannot become
insufficient through normal operation — it is structural.

### Evolving floor

The AVM adapts over time. The floor price can rise through area-preserving
raises or surplus-funded investment, permanently increasing the guaranteed
minimum value of every token. The market's sensitivity characteristics also
adapt — when supply contracts into the ramp or floor, the effective price
sensitivity changes, and the ramp–main boundary resets. These adaptations follow
the curve's geometry; the selected floor-raising policy controls when raises
occur.

### Credit liquidity

The AVM provides credit liquidity through the
[[avm-features--cash-advance-overview|cash advance]] facility. Because the
reserve already guarantees floor-value redemption for every token, advancing
that floor value against pledged collateral does not create a new liability — it
accelerates one that already exists. The same capital pool that backs the
pricing function also funds the credit facility. No separate lending pool or
external capital source is needed.
