---
title: Cash Advance Mechanics
summary:
  How the cash advance works — advance capacity sized by pledged collateral
  times floor price, collateral locking and automatic deleveraging as the floor
  rises, voluntary repayment, the reserve as the liquidity source, and why full
  utilization is generally rational.
tags:
  - avm-features
  - cash-advance
---

The cash advance lets m-token holders draw liquidity up to the floor value of
their pledged collateral, with no interest and no obligation to repay. The
reserve itself funds the advance — the same pool that backs redemptions. This
note covers the mechanics: how the credit line is sized, what happens to
collateral, how repayment works, where the liquidity comes from, and why full
utilization is generally rational.

---

## Advance capacity

A holder's advance capacity equals the total floor value of their pledged
collateral:

$$
\text{capacity} = n \cdot f
$$

where $n$ is the number of pledged m-tokens and $f$ is the current
[[avm--floor-raising|floor price]].

Because the floor is monotonically non-decreasing, advance capacity can only
grow for resting collateral. It increases in two ways: the holder pledges
additional m-tokens (increasing $n$), or the floor rises (increasing $f$). It
never decreases spontaneously.

The advance is denominated in the market's reserve asset — the same asset that
backs the [[avm--specification|price curve]]. If the market is denominated in
USDC, the advance is in USDC. If the market is denominated in gold, the advance
is in gold. The holder is not borrowing a foreign asset against their
collateral; they are unlocking liquidity already embedded in the floor
guarantee.

---

## Collateral locking

While an advance is outstanding, the protocol locks the minimum collateral
required to secure it. The locked quantity is:

$$
n_{\text{locked}} = \frac{\text{outstanding}}{f}
$$

Any pledged collateral beyond $n_{\text{locked}}$ remains free — the holder can
withdraw or redeem it.

When the floor rises, $n_{\text{locked}}$ decreases proportionally. An $x\%$
increase in the floor reduces the locked collateral by the same proportion,
because the same outstanding balance is now secured by fewer tokens at a higher
guaranteed value. This is **automatic deleveraging**: the holder's collateral
requirement shrinks without any action on their part, freeing tokens that were
previously locked.

No margin call or forced close-out is possible. The advance never exceeds the
floor value of the locked collateral, and the floor cannot decrease. The
position is solvent by construction.

---

## Repayment

Repayment is voluntary. The holder may repay any portion of the outstanding
advance at any time to unlock the corresponding collateral. There is no
repayment fee, no interest accrual, and no maturity date.

Reasons to repay include wanting to sell the underlying m-tokens, pledging them
elsewhere, or simply reducing the locked position. But there is no obligation —
the holder can carry the advance indefinitely at zero cost.

---

## Source of liquidity

The advance is funded directly from the market's reserve — the same capital pool
that backs all m-token redemptions.

This works because the advance is economically equivalent to a forward sale of
the collateral at the floor price, with an embedded option to repurchase. When a
holder takes an advance, they receive the floor value of their tokens — exactly
what the reserve would pay out if those tokens were redeemed at the floor. The
protocol retains the tokens as collateral, just as it would hold the reserve
after a redemption. The difference is that the holder keeps the right to reclaim
the tokens by repaying.

No external lender is needed, and no separate liquidity pool exists. The reserve
already guarantees redemption at the floor for every m-token in circulation, so
advancing floor value against pledged tokens does not create any new liability —
it accelerates one that already exists.

---

## Economics of full utilization

Ignoring the origination fee, it is generally rational to draw the advance to
its full capacity. The advance carries no interest and no obligation, making it
free liquidity with zero carrying cost. Capital drawn can be deployed elsewhere
while the holder retains full exposure to m-token appreciation above the floor.

This is especially powerful when the floor exceeds the holder's cost basis. If a
holder deposited 1.00 and the floor has since risen to 1.20, they can draw an
advance of 1.20 — recovering more than their entire initial outlay — while still
holding the m-token and benefiting from any further price appreciation. The
holder has effectively realized a profit without selling.

When the market price is close to the floor, the advance unlocks nearly all of
the holder's capital. If the market price is 1.00 and the floor is 0.90,
depositing 1.00 and immediately drawing the advance yields 0.90 back — the
holder's net capital commitment is only 0.10 for full m-token exposure. The
origination fee is the only friction.
