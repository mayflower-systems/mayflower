<!--
title: Mayflower
summary:
  "What Mayflower is and why it exists — a protocol that fixes the structural
  failures of crypto through four pillars: the AVM (reserve-backed markets with
  permanent floors), explicit parameter authority, earned-and-spent credit
  emission, and burn-auction revenue clearing."
tags:
  - mayflower
-->

Mayflower is a financial protocol built on Solana. It exists because crypto
broke its own promise. The technology that was supposed to eliminate
counterparty risk instead mass-produced it — unbacked tokens, forced
liquidations, concentrated control, worthless points. Mayflower returns to first
principles: every asset is fully reserve-backed, every guarantee is enforced by
math, and no participant depends on the good behavior of any other participant.

---

## The Amplifying Value Machine

The [AVM](Amplifying-Value-Machine) is the core of Mayflower. It is a deterministic pricing function
that is the market — no order book, no liquidity pool, no market maker. The
reserve equals the area under the price curve at all times. Solvency is a
mathematical identity, not a condition to be monitored.

**The floor locks in permanent gains.** Every Mayflower asset has a
[floor price](AVM-Floor-Raising) — a guaranteed minimum redemption value
backed by the reserve. The floor can rise. It cannot fall. When the floor goes
up, that gain is permanent for every holder. There is no giving it back.

**There is no rush to the door.** In conventional markets, a crash is a
coordination failure: everyone races to sell before the price hits zero, and the
last one out loses everything. In a Mayflower market, when the price reaches the
floor, it stops falling. Every holder redeems at the same price. There is no
advantage to panicking first. The floor eliminates the game-theoretic pressure
that turns sell-offs into death spirals. This alone changes what it means to
hold a crypto asset.

**Leverage without forced close-out.** The
[cash advance](Cash-Advance-Overview) lets holders draw liquidity
up to the floor value of their collateral — at zero interest, with no obligation
to repay. The advance can never exceed the floor, and the floor can never
decrease, so the position is solvent by construction at every moment. Forced
close-out is not a risk to be managed. It is structurally impossible. You can
take on leverage without any possibility of having your collateral seized.

**No shadowy counterparty.** The AVM has no liquidity providers, no external
market makers, no human discretion. The reserve is built by buyers and is the
counterparty to every sell. Every price, every fill, every reserve balance is
deterministically computable from the curve parameters. The market is fully
transparent because there is nothing to hide.

**The AVM absorbs anything.** The reserve asset can be dollars, bitcoin, ether,
solana, gold, tokenized equities — anything with a token on Solana. A Mayflower
market wrapping ETH creates an amplified value asset with a permanent floor
denominated in ETH. The AVM becomes a gravity well: a reserve that could absorb
the entire circulating supply of an asset, back every unit with a rising floor,
and unlock most of the capital back into liquid circulation through cash
advances. The bigger the reserve grows, the more stable and liquid it becomes.

**Amplified returns, bounded risk.** A Mayflower asset backed by bitcoin gives
you exposure to bitcoin with amplified upside — price movements on the bonding
curve are steeper than the underlying — while the floor caps your downside. This
is a different kind of leverage than a 10x margin position. Margin amplifies
both directions and liquidates you on a wick. The AVM amplifies upside, floors
your downside, and never liquidates. We want holding a Mayflower asset to feel
like owning real estate: an asset that appreciates, that you can borrow against
without risk of seizure, and that you hold for the long term because the
economics reward it. Not a store of value — a store of wealth.

---

## Credits and Allocation

[Mayflower credits](Mayflower-Credits) participate in emission routing and the
[clearing auction](Revenue-Clearing). A holder chooses which markets to support
by pledging credits to their channels. Each pledge remains committed to its
channel until its owner reduces it. The same credit cannot back multiple channel
pledges at once. Pledges require no referral membership and change allocation
immediately after elapsed emission settles.

Parameter authority is separate from channel weighting. Reserve records a root
administrator and an administrator for each deployment. The root creates
deployments; a deployment administrator controls its market policy, fees,
floor-raising policy, and emission decay. These operations require the recorded
authority's signature. Channel pledges do not execute parameter changes.

The emission algorithm constrains even an authorized decay change: it settles
accrued emission, then starts the next segment at the rate already in force. The
administrator may slow, accelerate, or pause decay, but cannot raise the
emission rate. Referral maximum depth and override rate are fixed when the deployment
is created. Individual parents control their admission offers and voluntary
sharing policies.

---

## Referral and Emission

Each Reserve deployment has one credit mint, one emitter, and an independent
referral network. Each of its markets has an emission outlet. The
[emission system](Credit-Emissions) determines the total reward budget,
channel weights divide it among outlets, and the
[Referral Network](Referral-Network) divides an outlet's allocation among
members. These stages allocate one budget without creating additional issuance.

Credit emission follows a whole-second geometric schedule whose annualized rate
can only stay flat or decrease. Cumulative entitlement is calculated from
immutable segment anchors, so pledge changes and checkpoint timing cannot
enlarge the schedule. Opening rate and decay are deployment configuration. A
constant positive decay has a finite ideal tail; pausing decay at a positive
rate permits indefinite issuance. A non-increasing rate therefore does not imply
an immutable lifetime-supply ceiling.

**Channel weighting.** Pledged credits set proportional outlet shares. An outlet
with no points records its allocation as unallocated, and later arrivals cannot
claim those intervals. Pledge changes settle the elapsed interval first, so
adding or withdrawing a pledge affects only subsequent allocation.

**Membership and points.** Reserve requires membership for emission points. Each
deposited AVA base unit contributes one point after enrollment, and each market
has a separate denominator. Nonmembers may use the AVM and pledge credits to
channels, but their collateral contributes no emission points. Enrollment
activates eligible collateral prospectively. Each source's own points enter its
outlet total once; downline aggregates are used only to calculate referral
entitlements.

**Referral allocation.** The [referral network](Referral-Network-Formal-Specification)
uses maximum depth $m$ and override rate $q$, with $mq<1$. A source at depth $d$
retains $1-dq$ of its reward attribution and reserves $q$ for each upline member.
With the reference values $m=2$ and $q=10\%$, own-point retention is 100%, 90%,
and 80%. A node's core weight is $(1-dq)p+qD$, where $p$ is its own points and
$D$ is its downline's points. The weights sum to the outlet's raw point
total.

Affiliates earn overrides from productive children and grandchildren without
matching their deposits. Splitting fixed capital among owned downline members or
siblings under an unchanged outside upline creates no extra combined core
entitlement. This guarantee does not extend to voluntary gifts or inserting
owned intermediaries above a new outside recruit. Higher layers retain more of
their own attribution and have more recruiting depth; admission terms determine
access to those positions.

**Mutualization.** A parent may share a chosen fraction of its core
earnings. The same fraction applies in every outlet. In each outlet, direct
children divide the shared amount by their own points in that outlet. Children
keep received gifts whole and never share them onward. Self-owned children can
receive gifts; their presence dilutes the offer to other children. A sharing
change applies in each outlet from that outlet's next settlement, while an
accepted upline is permanent.
Gifts redistribute existing earnings and never pay overrides again.

**Settlement.** Point balances settle under their previous values before a
change takes effect. Emission-weighted accounting preserves how
participation and rewards varied over time; referral sharing divides earned
rewards without creating another reward budget.
[Emission accounting](Emission-Accounting) explains how collected
rewards become issued credits.

Only final retained referral credit is payable: a deposit's gross source
attribution is not a second claim. After the deployment takes over its mint's
authority, successful claims are the issuance path. Any supply present at that
handoff remains existing supply, separate from the emission schedule. Channel
pledges use already-issued credits; pledging them creates no new rewards by
itself.

---

## Revenue Clearing

Market activity can generate fees, and protocol revenue can fund the
[clearing auction](Revenue-Clearing). Participants commit Mayflower credits for
shares of a reward pot. The protocol applies a time-dependent entry weight and
withdrawal penalty, then divides the net pot among surviving shares after the
epoch ends.

Remaining committed credits are irreversibly forfeited at finalization. Earlier
withdrawals return their unpenalized portion and forfeit the remainder.
Forfeited credits become unspendable, and a separate permissionless burn reduces
reported supply. The auction distributes rewards directly without a
secondary-market swap or a protocol-set credit price.

Each participant's reward depends on the funded pot, competing shares, entry
weights, withdrawals, and rounding. The clearing auction consumes credits; their
issuance through emissions is a separate process. Together, issuance and
forfeitures affect circulating supply.

---

## The Full Circuit

The four pillars are not independent features. They are a single economic loop.

The AVM creates amplified value assets with rising floors and built-in credit.
Fees fund revenue channels, where participants commit credits to the clearing
auction. Holders can also pledge credits to direct emission toward selected
markets. Within each outlet, eligible member deposits determine source
attribution, upline overrides reward recruitment, and optional mutualization
lets parents compete for children. Recorded authorities configure protocol
parameters within the constraints enforced by each algorithm. These mechanisms
connect market activity, reward allocation, recruitment, and revenue
consumption.

The resulting system combines reserve-backed markets, explicit administrative
authority, holder-directed emission, and voluntary revenue clearing. Its
accounting rules determine entitlements; participation and market activity
determine the economic value of those entitlements.
