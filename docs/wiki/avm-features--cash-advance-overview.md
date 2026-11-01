---
title: Cash Advance Overview
summary:
  What the cash advance is and why it matters — a non-recourse, zero-interest
  facility that lets m-token holders draw liquidity up to the floor value of
  their collateral, with no obligation to repay and no possibility of forced
  close-out.
tags:
  - avm-features
  - cash-advance
---

The Mayflower cash advance is a non-recourse facility that lets m-token holders
draw liquidity against their collateral at zero interest, with no obligation to
repay and no risk of forced close-out. It is funded directly from the market's
reserve — the same capital that backs all redemptions. The only cost is a flat
origination fee. See [[avm-features--cash-advance-mechanics|Mechanics]] for the
full specification.

---

## What it is

Holders pledge m-tokens as collateral and draw an advance up to the total
[[avm--floor-raising|floor value]] of their position. The advance is denominated
in the market's reserve asset — if the market is denominated in USDC, the
advance is in USDC; if in gold, the advance is in gold. The holder is not
borrowing a foreign asset against volatile collateral. They are unlocking
liquidity that is already guaranteed by the reserve.

The facility has three properties that distinguish it from every conventional
lending product:

**Zero interest.** There is no carrying cost. The advance accrues nothing over
time. The holder can carry it indefinitely — a day, a year, forever — at the
same cost as the moment they drew it.

**Non-recourse.** The protocol's only claim is the pledged collateral. If the
holder walks away without repaying, the protocol retains the locked m-tokens and
the matter is settled. There is no personal obligation, no deficiency judgment,
no recourse beyond the collateral itself.

**No forced close-out.** The advance can never exceed the floor value of the
collateral, and the floor can never decrease. The position is solvent by
construction at every moment. There is no margin call, no forced close-out, no
default trigger — because the condition that would trigger one is mathematically
impossible.

---

## Why these properties are possible

Traditional lending requires interest because a human lender needs compensation
for the time value of capital and the risk of default. Both of these disappear
in the Mayflower facility.

**No default risk.** The advance is capped at the floor value of the collateral.
The floor is a hard guarantee backed by the reserve — it cannot decrease. A
traditional lender prices default risk into the interest rate; here, default is
structurally impossible, so there is nothing to price.

**No human counterparty.** The protocol itself is the lender. It does not need
interest to incentivize lending — the reserve already exists and already
guarantees the floor. Advancing floor value against pledged tokens does not
create a new liability; it accelerates one that already exists (the reserve owes
floor value to every m-token in circulation). The protocol is simply disbursing
early what it would pay on redemption.

**No interest accrual.** This is also a design choice, not just a consequence.
Interest accrual would create a growing obligation that could eventually exceed
the collateral's floor value — reintroducing the possibility of insolvency and
the need for forced close-out. That contradicts Mayflower's core guarantee. A
flat origination fee avoids this entirely: the cost is fixed at the moment of
the advance and never grows.

---

## Fee structure

The only cost is an **origination fee** — a flat percentage of the amount
advanced, charged once at the time of the draw. There is no repayment fee, no
periodic interest, no penalty for early or late repayment (the concept of "late"
does not apply). The origination fee is set through the market's authorized fee
configuration.

---

## Why it is powerful

### Liquidity recovery

The advance lets holders recover most of the capital locked in their position.
If the market price is $1.00 and the floor is $0.90, a holder can deposit $1.00,
pledge the resulting m-token, and immediately draw $0.90. Their net capital
commitment is $0.10 for full exposure to a $1.00 position. They have recovered
90% of their liquidity while retaining all upside above the floor.

This is not leverage in the traditional sense — the holder is not borrowing a
different asset against volatile collateral. They are reclaiming liquidity that
the reserve already guarantees to them. The advance cannot be called, cannot
grow, and cannot default.

### Profit extraction without selling

When the floor rises above the holder's cost basis, the advance becomes a tool
for realizing profit without disposing of the asset. A holder who deposited
$1.00 and whose floor has since risen to $1.20 can draw an advance of $1.20 —
more than their entire initial outlay — while still holding the m-token. They
have extracted their original capital plus a $0.20 profit, and they remain fully
exposed to any further appreciation.

In traditional markets, this requires selling (triggering a taxable disposition)
or borrowing against the asset (incurring interest and forced close-out risk).
The cash advance achieves the same outcome with neither.

### Automatic expansion

As the floor rises, advance capacity grows automatically. The holder does
nothing — their credit line increases in lockstep with the floor, and if they
have an outstanding advance, the collateral required to secure it shrinks.
Capital that was locked gradually becomes free, available to withdraw or pledge
elsewhere. The facility improves over time by construction.

---

## A note on vocabulary

This facility is deliberately called a **cash advance**, not a loan. The
distinction matters both economically and legally.

A loan implies a creditor-debtor relationship: the borrower owes a debt, the
lender has recourse, interest accrues, and failure to repay constitutes default.
None of these apply here. There is no debt obligation, no recourse beyond the
collateral, no interest, and no concept of default. The holder can walk away at
any time, forfeiting only the pledged m-tokens.

The closest traditional analogue is a **repurchase agreement**: the holder
effectively sells their m-tokens to the protocol at the floor price and receives
an option to repurchase them by repaying the advance. But unlike a standard
repo, there is no fixed term, no coupon, and no obligation to repurchase.

The term "cash advance" — as in a merchant cash advance — captures the right
structure: a non-recourse disbursement against a future receivable (the floor
value), with a flat fee and no ongoing obligation.
