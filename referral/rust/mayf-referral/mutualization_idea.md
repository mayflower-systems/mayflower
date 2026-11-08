# Family Mutualization: Proposal for Review

Nov 8, 2026

## Summary

Each head sells slices of its own future override income to its children, and those slices share the family's income equally.

- **Joining is free.** Anyone can join under any node. Their deposits pay overrides up the lineage, as today.
- **Each family has a fixed number of shares.** The protocol fixes μ and S. A portion μ of every override a head earns goes into its family pot, which is divided into S equal shares.
- **The head owns all the shares at the start.** Unsold shares pay the head, so a head that sells nothing keeps all of its overrides.
- **The head sells shares to its children.** It chooses the price and the pace, and it receives the money. No share is ever issued beyond S, so a sale never dilutes earlier buyers.
- **The pot is split equally among the S shares.** Dividends are kept. They never pass further down and pay no override.

**Your two questions:**

1. **Does a new child pay with future credits?** Usually Bob pays the head's price upfront, with credits he earned from his own deposits and recruits, so he needs no outside money. A head could also sell on credit and keep the share's dividends until the price is paid. That would be safe, because only the head carries the risk.
2. **Does the parent get the join fee?** There is no join fee: joining is free. The head does get the money from selling shares, because it is selling its own income. That money is the head's reason to share.

**What changed from the first draft.** The first draft issued new shares and paid the money to the existing holders. An adversarial review found three fatal holes:

- **First-share camping.** Bots could buy the first share of every new node for almost nothing.
- **First-share race.** Once a family earned, the first buyer took μ of the head's income for a price that was burned.
- **Dilution.** New shares sold below their lifetime value let whoever ran the most accounts drain earlier holders.

A fixed supply owned by the head closes all three.

## Goals

The referral tree should reward people for bringing real capital and real participants, and make some families worth more to join than others.

- **Alice, the creator.** She has no money but makes good content. Rich people use her link, and she benefits from their deposits.
- **Bob, the passive earner.** He has little money and wants passive income. He compares links, finds Alice, and sees that joining her family will pay him without further work.
- **A stronger network.** Incentives should follow skin in the game: capital at risk, or credits earned and committed.
- **Real participants over sybils.** 1,000 real participants are worth far more than 100,000 fake accounts. No reward should grow just because someone made more accounts.
- **Families that eat together.** When a rich grandchild arrives, the head's whole family benefits, including uncles like Bob.
- **Horizontal competition.** Seats in different families at the same depth should differ in value, so links are not interchangeable.

## Design rules

Five rules shape the design. Each one comes from an attack found while testing earlier versions.

1. **Whatever a new account gets for free, a sybil gets for free.** So a claim on a family's income must be bought from whoever owns it. Joining the tree can be free only because a node with no deposits earns nothing.
2. **Rewards in proportion to capital are safe; rewards per account are not.** Splitting a deposit across ten accounts changes no override. No reward can grow with the number of accounts, so headcount itself cannot be rewarded without identity.
3. **Never create new claims on income that others already own.** Issuing new shares dilutes earlier holders, and any price formula that undervalues a share moves their income to the buyer. With a fixed supply, a sale moves income only from the seller to the buyer.
4. **A promise the promiser can break is worth nothing.** Children cannot leave the tree. So the protocol fixes μ and S, and a sold share cannot be taken back.
5. **When anyone settles must not move credits.** Income arrives in random chunks, whenever a node is settled. Each chunk is exactly what accrued since the last settle. So splitting each chunk equally among the S shares is exact, provided a share changes hands only right after the head is fully settled.

## The mechanism

A family is a head plus the children who hold shares in its pot. Membership in the tree is free; a share is bought from the head.

| Role        | Who                         | Pays                                  | Receives                                                                                              |
| ----------- | --------------------------- | ------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Head        | Any node with children      | Nothing extra                         | (1 − μ)·q on every unit in its subtree, the income of its unsold shares, and the money from each sale |
| Member      | Any child node              | q on its points to each upline member | Its own emission, and overrides from its own subtree                                                  |
| Shareholder | A member that bought shares | The price, once per share             | 1/S of its parent's family pot per share                                                              |

1. **Joining.** Any node may join under any node above the maximum depth, for free. A tiny burned fee can deter spam. Top-level seats stay scarce and priced by the protocol, because they are the only way to avoid paying overrides.
2. **Overrides.** Unchanged. A member at depth d pays q of its points to each of its d upline members.
3. **The family pot.** Of every override a head earns, μ·q goes to its pot and (1 − μ)·q to the head. The pot is divided into S equal shares. So μ is the most of its override income a head can ever sell. It always keeps (1 − μ)·q, which keeps it working for the family after it sells.
4. **Shares.** At creation, the head holds all S shares. It may list shares for sale to its children, at a price and pace it chooses, and a child may hold several. The buyer pays upfront in credits, for example from the credits its node has earned. A head could also sell on credit, an open question, keeping the share's dividends until the price is paid.
5. **Dividends.** As income arrives, each share receives 1/S of the pot. Dividends stay with the holder's node. They never pass further down and pay no override.
6. **Permanence.** A sold share stays sold. The head cannot take it back, issue more, or change μ.

## Token flows

Credits reach a family pot from one place: μ of every override the head earns. A share sale moves money from the buyer to the head, and never touches the pot.

&#91;embedded content: token flows · overrides fill the pot; the head sells fixed shares\]

The head always keeps (1 − μ)·q. Unsold shares pay the head too, so a head that sells nothing keeps all of q. A sale moves one share and its income from the head to the buyer, and leaves every other holder's share untouched.

## Why each player behaves as intended

Each player's best move is the one the network wants: bring deposits, recruit real people, and trade shares at prices both sides accept.

- **Alice, the head.** She earns (1 − μ)·q on every unit in her subtree, plus the income of the shares she has not sold. Selling shares turns future income into credits now, which a creator with no money needs. Every buyer becomes someone who wants her family to grow.
- **Elon, the rich fan.** He joins under Alice for free and pays q to each upline member on his deposits. He may buy shares from Alice to win back part of his overrides, at a price they both accept. He accepts the overrides out of loyalty to Alice, and because a top-level seat costs more.
- **Bob, the passive earner.** He joins under Alice for free and deposits what he has. He earns credits from his own deposits and recruits, and buys a share once he can afford Alice's price. Early shares are cheap because Alice has no track record yet, so his bet pays off if her family grows.
- **Shareholders.** Every deposit anywhere in Alice's subtree raises their dividends, so they recruit for the whole family. Later sales never dilute them, because the supply is fixed.
- **Sybils.** A node with no deposits earns nothing. Only their owners sell shares, at prices the owners set, so there is nothing to take for free. Splitting deposits across many nodes changes no override.

No design can pay Bob for nothing without also paying sybils for nothing (rule 1). Bob's edge is information and timing: he spots a rising creator early, while her shares are still cheap.

## Worked example

Bob pays 10 credits for one early share, and Elon's arrival lifts its dividend from 0.025 to 25 credits an hour.

Assumptions: one large lane where each unit earns 1 credit an hour throughout, q = 5%, μ = 50%, and S = 10 shares per family.

1. Alice takes a top-level seat and deposits nothing. She holds all 10 shares in her family.
2. Bob joins under Alice for free and deposits 10 units. Alice sells him one share for 10 credits, and she receives them.
3. Elon joins under Alice for free and deposits 10,000 units.

| Credits per hour                       | Before Elon | After Elon |
| -------------------------------------- | ----------- | ---------- |
| Alice's overrides (5% of her subtree)  | 0.5         | 500.5      |
| Alice keeps outright (half)            | 0.25        | 250.25     |
| Family pot (half)                      | 0.25        | 250.25     |
| Each share (a tenth of the pot)        | 0.025       | 25.025     |
| Alice's total (outright plus 9 shares) | 0.475       | 475.475    |
| Bob's total (his own 9.5 plus 1 share) | 9.525       | 34.525     |
| Elon keeps (95% of 10,000)             | –           | 9,500      |

**Alice sells a second share after Elon arrives.** Say she asks for 30 days of its dividend: 720 × 25.025 = 18,018 credits. She receives all of it. Bob's share still pays 25.025 an hour, because the supply is fixed.

**Timing is Bob's edge.** Had Bob arrived after Elon, his share would have cost about 18,000 credits, far beyond his means. He profited because he bought before the family's income arrived. Alice profited too: she raised credits when she had none.

## How shares are priced

With a fixed supply, a share's price matters only to its seller and its buyer, so the protocol needs a simple sale mechanism and no valuation formula.

**Why the earlier requirements fall away.** When new shares were issued, an underpriced share took income from the existing holders. The protocol therefore had to price shares from the pot's income. A purely time-based auction let buyers take most of a sudden windfall: in one test, Bob kept about 12,000 of the 180,180 credits his share was worth. Now a sale moves income only from the seller to the buyer. If Alice sells too cheaply, she alone loses.

**What the sale mechanism needs:**

1. **The seller sets the terms.** A head lists shares at a fixed price, or as a time-based auction that falls from an ask to a reserve. Each sale can raise the next ask by a set factor, so prices climb as shares sell.
2. **The buyer sets a limit.** The buyer names the most it will pay, and the purchase fails above that.
3. **The head is settled first.** A sale settles the head in every lane before the share changes hands, so each side gets exactly the income from its own time.
4. **Clients show the history.** Buyers should see the pot's income over time and how much of it comes from one depositor. That is their defence against a head who inflates its income briefly before selling.

## Risks and open questions

Under these rules, settlement timing cannot move credits. Five market risks remain, five decisions are open, and the fixed-supply version still needs its own adversarial review.

**Risks:**

- **Pump and dump.** A head or its friends can deposit heavily for a while, sell shares at high prices, then withdraw. Buyers' limits and the income history clients show are the only defence.
- **A head can move on.** After selling shares, Alice could steer new recruits to a second node whose shares she still owns. Her retained (1 − μ)·q softens this, but buyers should price it in.
- **Deep links cost more.** A fan who joins a deep creator pays q to every upline member, so fans prefer shallow links. The price of top-level seats decides how strong this pull is.
- **Shares outlive activity.** A holder who withdraws everything keeps its dividends forever. It paid for them, so this is fair, but the share no longer reflects skin in the game.
- **Legal.** Selling slices of future income for upfront credits may count as a security, and rewards that flow up a referral tree draw pyramid-scheme scrutiny. Have a lawyer review the design before launch.

**Open questions:**

- [ ] Values of μ and S. The example uses μ = 50% and S = 10.
- [ ] May holders resell shares to other children? That would give Bob a way out.
- [ ] May a head sell on credit, keeping a share's dividends until it is paid?
- [ ] Is a cap of 16 lanes acceptable?
- [ ] Does joining need a burned fee against spam, and how large?

## Changes from the current kernel

The change replaces per-lane, parent-chosen sharing with one family pot per head, split into a fixed number of shares. It also needs a cap of 16 lanes.

| Area                           | Today                                                                  | Proposed                                                                                |
| ------------------------------ | ---------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| Joining under a parent         | Pays the parent's fee, which rises with each join                      | Free, apart from a tiny burned fee                                                      |
| Parent settings                | `offer` and `sharing` on each node, with `set_offer` and `set_sharing` | Share listings: a price, or auction terms                                               |
| Sharing rate                   | Chosen by each parent                                                  | μ and S, network terms next to q                                                        |
| Where shared credits come from | A portion of everything the parent earns, lane by lane                 | μ of the head's overrides, from every lane, into one pot                                |
| Who receives them              | Children who hold the lane, by balance                                 | Holders of the S shares, 1/S each; unsold shares pay the head                           |
| Per-lane sharing fields        | `children`, `gift_index`, `parent_gift_index_seen`, `sharing_seen`     | Removed                                                                                 |
| Per-head fields                | None                                                                   | Pot index, pot remainder, unsold shares, listings                                       |
| Per-member fields              | None                                                                   | Shares held, and the pot index last seen                                                |
| New actions                    | –                                                                      | List shares; buy a share: settle the head in every lane, pay the seller, move the share |
| Lanes                          | Up to 10,000, one loaded per action                                    | Up to 16, so a sale can settle the head in all of them                                  |

Top-level seats stay priced by the protocol, as today.
