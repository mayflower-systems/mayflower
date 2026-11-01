---
title: Emission Schedule
summary:
  A whole-second geometric schedule defines the non-increasing credit emission
  rate and cumulative entitlement from immutable segment anchors.
tags:
  - tokenomics
  - emission
  - math
---

The [[credits-emissions|credit emission schedule]] determines the total reward
entitlement available over time. It is independent of channel pledges, channel
points, and referral relationships: those determine how the scheduled amount is
divided. The opening annual rate and decay coefficient are supplied when a
deployment is created.

## Rate and cumulative emission

Let $R_0\ge0$ be the annualized rate at the start of a segment, $D\ge0$ the
annual decay coefficient, and $S=31{,}536{,}000$ the number of seconds in a
365-day year. The ideal per-second retention is

$$q=\max\left(0,1-\frac{D}{S}\right).$$

After $n$ complete seconds, the annualized rate is $R_n=R_0q^n$. During second
$k$, the schedule attributes $R_k/S$. Cumulative entitlement since the segment
began is

$$
E_n=\frac{R_0}{S}\sum_{k=0}^{n-1}q^k=
\begin{cases}
nR_0/S,&q=1,\\
R_0(1-q^n)/(S(1-q)),&0\le q<1.
\end{cases}
$$

With $D=0$, the annual rate stays constant and cumulative entitlement grows
linearly. With positive decay, the rate declines. If the per-second decrement is
at least one, $q$ is clamped to zero: the initial one-second tranche accrues and
the subsequent rate is zero.

For example, at $R_0=1{,}000$ credits per year and $D=0.15$, the ideal rate
after one year is approximately 860.71 credits per year, and about 928.61
credits have accrued during that year. A 15% coefficient produces an
approximately exponential decline; it does not mean exactly 15% of the opening
rate disappears in the first year.

## Changes to decay

The deployment administrator can change $D$. A change first settles the existing
segment, then starts a new segment at the annual rate already in force.
Increasing $D$ makes subsequent decay faster; reducing it slows the decline;
setting $D=0$ pauses decay at the current rate. Since $D$ is nonnegative and the
new segment begins at the settled rate, these changes cannot raise the
instantaneous emission rate or rewrite earlier accrual.

The intended opening policy is an annual rate equal to the then-current supply
of M and a 15% annual decay coefficient. These are intended configuration
choices, not values inferred by the schedule. Actual opening rate and decay are
supplied explicitly.

With a fixed positive $D$ that yields $0<q<1$, the ideal remaining emission is
$R_0/D$. This limit is conditional on keeping that decay fixed. Holding a
positive rate at $D=0$ instead continues issuance indefinitely at that rate; the
non-increasing rate alone does not establish a fixed lifetime-supply ceiling.

## Anchors and precision

Each segment's cumulative entitlement is evaluated from its fixed start and
compared with the amount already recorded. Splitting an unchanged interval into
more checkpoints therefore cannot enlarge the schedule. A decay change closes
one segment and establishes the next anchor; pledge and point changes do not
reanchor it.

The actual schedule uses fixed-point arithmetic. For positive decay, the
per-second decrement rounds upward, and the calculated rate and cumulative
entitlement round downward. These directed approximations preserve
non-increasing, conservative accrual. The formula above describes the ideal
geometric schedule; actual represented amounts follow its fixed-point rules.
