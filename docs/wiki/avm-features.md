---
title: AVM Features
summary:
  Optional features built on top of the core AVM — add-ons that extend the
  pricing mechanism without being required for its operation.
tags:
  - avm-features
---

Optional features that extend the [[avm|AVM]] beyond its core pricing mechanism.
These are add-ons that individual markets may enable or disable. The core AVM
(issuance, redemption, floor raising, contraction) functions without any of
them.

---

## Notes

- [[avm-features--call-options|Call Options]] — Perpetual call options with
  strike equal to the floor. Mints tokens at floor price, translates the curve
  rightward, preserves the reserve invariant.
- [[avm-features--floor-investment|Floor Investment]] — Surplus-funded floor
  raises that preserve the live price while allowing the floor/ramp geometry to
  consume old boundaries.
- [[avm-features--cash-advance-overview|Cash Advance Overview]] — What the cash
  advance is, why zero interest and non-recourse are possible, fee structure,
  and why it is powerful.
- [[avm-features--cash-advance-mechanics|Cash Advance Mechanics]] — How advance
  capacity is sized, collateral locking and automatic deleveraging, repayment,
  the reserve as liquidity source, and economics of full utilization.
