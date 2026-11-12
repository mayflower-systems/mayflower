# mayf-referral

`mayf-referral` is a Rust library for distributing rewards through a multi-level
referral network. Members earn credits from eligible asset balances. The people
above them in the tree earn referral commissions, and can offer some of those
commissions back to their immediate children.

This connects recruitment to ongoing participation: a referral relationship earns
commissions as the referred member and their descendants hold eligible positions.
Parents can compete for members through admission prices and reward-sharing
offers. A member can earn from their own position without recruiting anyone, or
earn commissions from their descendants without holding an asset themselves.

The library handles the accounting behind these relationships. Balances change,
members stake and withdraw credits, and parents revise their offers. Previously
earned rewards must remain with the people who earned them, without recalculating
the entire network on every change.

## How rewards flow through the tree

Members below the top level choose one permanent parent. The **upline** is the
chain of ancestors above a member; the **downline** contains all their descendants.
A network fixes its maximum depth and the commission paid to each earning ancestor.

Eligible assets are organized into **lanes**. Each lane has its own credit budget,
distributed in proportion to the balances held in that lane. For example, a BTC
lane and an ETH lane account for their assets separately; they do not compare
raw BTC quantities with raw ETH quantities. A balance can represent an internal
application position.

An asset holder's share of the lane budget is then divided between the holder
and their ancestors. If each ancestor receives a fraction `q` and the holder has
`d` earning ancestors, the holder keeps `1 - d*q`. Commissions come from that same
allocation, so adding referral levels does not create extra rewards.

Suppose Alice is a top-level member who referred Bob, and Bob referred Carol.
With a 10% commission, every 100 credits attributed to Carol's asset balance
are divided like this:

| Member | Reason                                 | Credits |
| ------ | -------------------------------------- | ------: |
| Carol  | Holds the eligible position; keeps 80% |      80 |
| Bob    | Carol's parent; receives 10%           |      10 |
| Alice  | Carol's grandparent; receives 10%      |      10 |

Each ancestor receives the same commission, regardless of distance. Top-level
members have no earning ancestor above them. The network requires `H*q < 1`,
where `H` is its maximum depth, so even the deepest member retains a positive
share of their own position's rewards.

The lane budget is shared: new eligible balances also dilute existing balances'
share. Recruiting earns a commission on the new participation; it does not
guarantee an increase in absolute income.

## Sharing commissions with children

A parent can offer a percentage of its referral commissions to its immediate
children. Each child's share is proportional to the **already-issued credits it
holds in escrow** with that parent. This gives parents a way to use their referral
income to attract members, and gives children a way to participate in their
parent's referral income.

Continuing the example, suppose Alice keeps her commissions, Bob shares half of
his, and Carol is Bob's only child with escrowed credits. Bob gives Carol 5 of his
10 credits. The final allocation is Alice 10, Bob 5, and Carol 85: still 100 credits
in total. With multiple participating children, Bob's shared amount is divided
by their relative escrow balances.

Only referral commissions fund this sharing. A parent's own-position rewards
remain theirs, and received gifts stay with the recipient rather than being
shared again. If no child has credits in escrow, the parent keeps all commissions.
A child can receive shared rewards even without an eligible asset balance.

Escrow remains the child's withdrawable principal. Earnings do not automatically
increase it. Parents may change their sharing percentage for future earnings;
the offer is not a permanent right to a portion of their income.

## Joining the network

An ordinary member joins under an existing parent at that parent's posted credit
fee, which may be zero. The fee goes to the parent and is separate from any
credits the child later puts into escrow. Parents can revise their fee and choose
an automatic price increment after each successful join.

Top-level membership uses an authorized auction award supplied by the application;
activation specifies the credit amount the application must burn. The parent
relationship is permanent, including after a member withdraws all their balances.

## Accounting without walking the whole tree

The library records cumulative rewards per unit of eligible balance. A member's
checkpoint identifies the rewards already accounted for in their accrued balance.
Each ancestor also tracks its aggregate downline balance, so commissions can be
calculated without loading individual descendants. A similar index distributes
a parent's shared commissions among children with credit escrow.

Changes always settle earlier earnings under the earlier terms:

- A balance change settles the member and their ancestor path before updating
  eligible balances.
- A sharing or escrow change settles all of the parent's source lanes before
  applying the new percentage or stake.
- Collection pays already-accrued net credits, preserving individual fractions
  for later collection.

This prevents new balances from claiming old rewards and new offers from changing
what people already earned. Work is bounded by the configured tree depth and
lane count, rather than the number of members. The current implementation
supports up to sixteen lanes.

## Using the library

The main records are a `Protocol` for network terms, a `Lane` for each asset's
reward accounting, a [`Node`](src/tree/node.rs) for membership and family sharing,
and a [`LaneNode`](src/tree/lane_node.rs) for a member's balance and downline total
in one lane.

The [`program` module](src/program.rs) exposes actions for joining, updating
balances and offers, staking credits, settling rewards, and collecting them.
Each action takes state and returns a receipt containing the next state and any
required transfers, burns, or payouts. The embedding application authenticates
inputs and applies the whole receipt atomically.

For reward settlement, the application supplies each lane's trusted cumulative
earnings as a precise fixed-point number. The library consumes only the increase
since its previous checkpoint. The application must bring these totals current
before changing balances or family terms, so past earnings use the correct
eligibility. Empty-lane earnings are recorded as unallocated, not saved for the
next depositor.

The [runnable example](examples/emissions.rs) follows Alice, Bob, and Carol through
joining, holding positions in two lanes, staking credits, sharing commissions,
and collecting their rewards. From the workspace root:

```bash
cargo run -p mayf-referral --example emissions
cargo doc -p mayf-referral --open
```

See [SPEC.md](SPEC.md) for the economic rules and [source documentation](src/lib.rs)
for numerical precision, rounding, and the application's responsibilities.
