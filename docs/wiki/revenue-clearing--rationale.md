---
title: Rationale
summary:
  Why revenue clearing uses a public auction, how its entry and exit rules
  address information and timing, and what fairness means in this model.
tags:
  - revenue-clearing
  - tokenomics
---

[[revenue-clearing|Revenue clearing]] distributes a funded reward pot in
exchange for clearing tokens. Its rules make participation, withdrawal, and
settlement computable from public state. The central design problem is how to
run an auction when participants can see and react to one another's actions
throughout the auction.

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

An application with revenue and a token can buy that token on a market and burn
it. That route depends on market liquidity and available prices and can incur
trading fees, slippage, and transaction-ordering costs. Revenue goes to the
holders who sell into the buyback.

Clearing changes the exchange: funders supply a reward pot, and participants
commit clearing tokens for shares of that pot. Remaining principal is
irreversibly forfeited at finalization; withdrawal forfeits a time-dependent
portion. The reward is allocated directly among the surviving shares, without a
secondary-market swap or a price oracle in the settlement rule. A participant
decides how much to commit based on the reward it expects and the other
commitments it expects to compete against.

In Mayflower, [[credits|Mayflower credits]] serve as the clearing asset and
protocol revenue can fund rewards. Clearing provides a way to spend credits to
claim a portion of a funded pot. Its effect on circulating supply combines with
credit issuance, which follows a separate schedule.

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
affect the outcome. See [[revenue-clearing--time-weight|Time Weight]] for the
lot-ordering and rounding details.

## Why reward earlier entry?

For elapsed time $t$ in an epoch of duration $T$, the ideal entry weight is

$$w(t)=1+b\left(1-\frac{t}{T}\right),\qquad 0\le b\le2.$$

A deposit earns its principal multiplied by this weight in shares. At $b=1$, the
weight starts at 2 and approaches 1 at the deadline. At $b=0$, every deposit has
the same weight. The highest permitted bonus gives a starting weight of 3.

The early bonus makes it worthwhile to commit while the eventual pot and other
commitments are less certain. Waiting can reveal more information, but each
token earns less weight as time passes. A linear curve spreads this tradeoff
evenly through the epoch: for a fixed $b$, ideal weight decreases at a constant
rate. A larger $b$ places more emphasis on early commitment; a smaller $b$ gives
more equal treatment across entry times.

New deposits increase total shares and dilute existing shares of a fixed pot.
Earlier weight compensates early entrants for committing before those later
actions are known; it does not prevent later participation or dilution. The
reward per token also depends on the final pot and any withdrawals.

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

The ideal result is tempered by discrete accounting. Each withdrawal handles one
lot and its returned principal rounds down to whole units; partial share removal
rounds up to the share grid. Dividing deposits among positions can change the
rounding and the shares surrendered. The
[[revenue-clearing--formal-spec|formal specification]] gives the exact
accounting rules.
