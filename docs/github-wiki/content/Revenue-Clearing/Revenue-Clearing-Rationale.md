<!--
title: Revenue Clearing Rationale
summary:
  Why revenue clearing uses a public auction, how its entry and exit rules
  address information and timing, and what fairness means in this model.
tags:
  - revenue-clearing
  - tokenomics
-->

[Revenue clearing](Revenue-Clearing) distributes a funded reward pot in exchange
for clearing tokens. Its rules make participation, withdrawal, and settlement
computable from public state. The central design problem is how to run an
auction when participants can see and react to one another's actions throughout
the auction.

## Why a public blockchain is difficult for auctions

A conventional sealed-bid auction relies on bids remaining hidden until a common
reveal, and on an auctioneer or another mechanism applying the rules. On a
public blockchain, actions and their ordering can be visible and economically
valuable. A participant can wait to see the pot or other bids, submit a
transaction near the deadline, use multiple addresses, or react to a large
apparent commitment. A transaction's place in an ordered block can change what
others know and whether it lands before the deadline. Reordering and
front-running can therefore be economically valuable: the participant who acts
on another bid may change the share total before settlement.

There is no single ordering that gives every participant identical information
and opportunity. Even a rule that processes the first accepted transaction has
to use an ordering produced by the blockchain. Revenue clearing therefore does
not attempt to make all participants simultaneous or equally informed. Instead,
it specifies the consequences of entering and leaving at each time, then settles
all surviving shares by the same formula. The entry and exit curves price timing
choices; they do not remove the blockchain's transaction ordering.

## Why distribute revenue through an auction?

AVM markets assess fees in their respective base reserve assets. In the intended
integration, each AVM action transfers its protocol-fee allocation directly into
the corresponding clearing pot in the same transaction. A BTC-denominated pot
stays in BTC; a SOL-denominated pot stays in SOL. The protocol does not need to
sell those assets into one settlement currency before distributing them.

Participants commit [Mayflower credits](Mayflower-Credits) for weighted shares
of a chosen pot. The algorithm uses reward quantities and credit shares, without
a dollar-price oracle or an administrator setting exchange prices. Participants
make their own valuations when deciding where and how much to commit. Those
choices can use outside price information, but settlement does not depend on
that information being supplied to the protocol.

The auction distributes the pot proportionally among surviving shares. It does
not select one winning bidder or certify a true market price. Remaining credit
principal is burned at finalization; a withdrawal burns its penalty immediately.

## Why use credits as the common bidding unit?

Credits give participants one bidding unit across pots of different reserve
assets. Their scheduled allocation through the referral network connects the
ability to bid with eligible asset participation. Burning committed credits
consumes that bidding resource without creating another pot of bid proceeds for
the protocol to sell or distribute.

An external token could also serve as a clearing token; a new token is not a
requirement of auctions in general. Mayflower chooses credits to connect its
participation-allocation rules to fee distribution without selecting one
external asset as the common payment currency. Neither credits nor gauge weights
are dollar-value measurements. Gauges choose relative emission allocations, and
bidders choose how to spend the resulting credits.

Avoiding a liquidation desk and avoiding a price oracle are separate design
properties. An oracle concerns valuation inputs; custody concerns control over
assets. Clearing still holds assets in program escrow under its settlement
rules. The actual configuration and upgrade controls of a deployment must be
specified separately, as described in [Protocol Scope](Protocol-Scope).

The clearing mechanism also accepts funding from other applications. Its
allocation rules depend on accepted funding, regardless of the source. Credit
issuance follows a separate schedule; burning credits does not replenish it.

## What makes the allocation fair?

The mechanism applies the same rules to every accepted action:

- **Public terms.** The epoch duration, entry curve, and exit curve are fixed
  for the epoch. A participant can calculate the consequences of an action from
  those terms and the current state.
- **Time has an explicit price.** Earlier commitment earns more weight when the
  entry bonus is positive. Waiting provides time to observe the pot and
  competitors while earning less weight per token.
- **Exit has an explicit cost.** A withdrawal forfeits a fraction of its
  requested principal according to time since the epoch began. Creating a new
  deposit or address does not reset that clock.
- **One rule for shares.** Each surviving share receives the same fraction of
  the final pot before whole-unit rounding. Claims use the same fixed totals, so
  claim order does not change payouts.
- **No identity requirement.** The allocation is based on deposited tokens,
  their entry times, withdrawals, and surviving shares. It does not require a
  registry to decide which addresses belong to the same person.

This fairness is about the rule applied to each action, not equal outcomes.
Early and late participants get different weights by design. A participant with
more information, capital, or influence over transaction timing can make
different choices. Multiple addresses do not create a special claim by
themselves, but rounding and the way deposits are divided among positions can
affect the outcome. See [Time Weight](Revenue-Clearing-Time-Weight) for the
lot-ordering and rounding details.

## Why reward earlier entry?

For elapsed time $t$ in an epoch of duration $T$, the ideal entry weight is

$$w(t)=1+b\left(1-\frac{t}{T}\right),\qquad 0\le b\le2.$$

A deposit earns its principal multiplied by this weight in shares. At $b=1$, the
weight starts at 2 and approaches 1 at the deadline. At $b=0$, every deposit has
the same weight. The highest permitted bonus gives a starting weight of 3.

The early bonus offers more shares while the eventual pot and other commitments
are less certain. Whether that makes entry worthwhile depends on the
participant's expectations. Waiting can reveal more information, but each token
earns less weight as time passes. A linear curve spreads this tradeoff evenly
through the epoch: for a fixed $b$, ideal weight decreases at a constant rate. A
larger $b$ places more emphasis on early commitment; a smaller $b$ gives more
equal treatment across entry times.

New deposits increase total shares and dilute existing shares of a fixed pot.
Earlier weight favors early entrants who commit before those later actions are
known; it does not prevent later participation or dilution. The reward per token
also depends on the final pot and any withdrawals.

## Why charge for withdrawal?

A large early commitment may discourage others by making the expected reward per
share look small. If that commitment could be removed freely near the deadline,
the participant could later recover the tokens after discouraging competition.
The exit penalty makes such a late reversal costly.

For a withdrawal at elapsed time $t$, the ideal penalty fraction is

$$p(t)=\left(\frac{t}{T}\right)^k,\qquad k\in\{1,2,3,4\}.$$

The reference $k=2$ gives a 25% penalty halfway through and 81% at 90% elapsed.
Larger exponents make early and middle withdrawals cheaper and concentrate the
penalty closer to the deadline; $k=1$ makes the ideal penalty linear. The clock
starts with the epoch, so a fresh lot does not create a cheaper exit at the same
time.

The entry and exit rules address different decisions: entry weight rewards
earlier commitment, while the exit curve prices a later decision to reverse it.
Their effect on a strategy depends on the amount committed, other participants'
actions, reward funding, and the timing of any withdrawal.

## Why use separate deposit lots?

Each deposit earns shares at its own entry weight. Keeping deposits as separate
lots preserves those shares instead of blending their entry times. Withdrawals
take the newest surviving lot first. Since entry weight does not increase over
the epoch, that lot has the lowest or a tied weight. In ideal arithmetic, this
ordering removes the fewest shares for a given withdrawal and preserves older,
higher-weight lots.

The ideal result is tempered by discrete accounting. One withdrawal can span
several lots, and its total returned principal rounds down once to whole units;
partial share removal rounds up to the share grid. Splitting that request into
separate withdrawals can reduce the total refund. Dividing deposits among
positions can change rounding and the shares surrendered. For a fixed final
share entitlement, however, splitting it across claimants cannot increase the
combined payout: the sum of rounded-down claims is at most the combined claim
rounded down once. The
[formal specification](Revenue-Clearing-Formal-Specification) gives the exact
accounting rules.

## What the auction does not guarantee

**Competition must come from actual participants.** With no competing shares,
the sole positive-share bidder receives the entire tracked pot, even if its
accepted bid is only one clearing-token base unit. Creating several addresses
under the same owner's control does not create independent competition. The
auction has no minimum credit price or reserve-price rule, and a burn does not
by itself establish the token's market value.

**Time weights limit an advantage; they do not eliminate late entry.** The
maximum start-of-epoch weight is 3, while the deadline limit is 1. A
sufficiently large later deposit can therefore outweigh earlier deposits. The
earlier bonus does not remove information advantages or control over transaction
inclusion.

**Rewards must be funded.** Already accepted funding is held for the epoch, but
a promise to add revenue later is not an escrow balance. Finalization burns
remaining principal even if no rewards were supplied. Funding accumulated
revenue before inviting bids makes that amount observable; it does not fix the
number of competing shares or guarantee a bidder's eventual payout.

**Accounting rules do not prove revenue growth.** In the wider Mayflower system,
[referral allocation](Referral-Network) determines who earns credits, and
[family marketing](Mutualization) redistributes some of those earnings. Clearing
gives credits a use against funded rewards. These are separate flows: credit
issuance or burning cannot itself create the reward asset. Continuing economic
value depends on the activity that supplies revenue and on participants choosing
to spend credits for it.
