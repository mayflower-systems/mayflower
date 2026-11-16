# Mayflower referral network bundle

This is a standalone companion to the broader Mayflower incentive bundle. It uses the same visual style and focuses on the referral network’s two allocation stages: core rewards from eligible holdings, followed by optional sharing of referral commissions.

## Reading order and files

| File stem | Purpose |
| --- | --- |
| `00-master` | One shared tree, a core allocation, and an optional family pool. |
| `01-earn-through-the-tree` | How a holding’s credit attribution divides among its holder and earning ancestors. |
| `02-share-with-the-family` | How a parent’s sharing percentage and children’s credit escrow determine family allocations. |
| `03-benefit-across-branches` | Why someone can benefit from participation in a sibling’s branch. |
| `04-choose-a-link` | Ordinary admission, top-level auctions, family escrow, and permanent parent links. |

Each chart is supplied as an editable SVG and a high-resolution PNG. The master is 1600 × 1120 in SVG and 3200 × 2240 in PNG. Companion charts are 1080 × 1080 in SVG and 2160 × 2160 in PNG.

- **`mayflower-referral-network.drawio`** contains all five charts as editable diagrams.net pages. Text, boxes, and connectors remain separate objects.
- Each **SVG also embeds its own diagrams.net page**, matching the editable format of the original supplied flowchart. Ordinary SVG text and shapes remain available to vector editors.
- **`companion-charts-preview.png`** gives a compact overview of the four companion charts.
- **`referral-network.md`** is a public-facing explanation of how the mechanism works and why its parts fit together.

Lead with the master chart. Use the numbered charts as a carousel or alongside the corresponding parts of the explanation. Charts 1–3 deliberately continue the same worked example.

## The worked example

The membership links are:

```text
Protocol root — non-earning
└─ Alice — auctioned earning seat, depth 0
   ├─ Bob — ordinary member, depth 1
   │  └─ Dana — ordinary member, depth 2
   └─ Carol — ordinary member, depth 1
```

The diagrams show this branch, not every member of the network. For the example:

- Dana holds 10% of one lane’s eligible balance during an interval allocated 3,000 credits. Her gross attribution is therefore 300 credits.
- The other eligible positions are outside Alice’s branch. Alice, Bob, and Carol have no own-asset rewards in this example. The displayed commission totals come entirely from Dana’s source.
- Each earning ancestor’s commission is 10% of the original attribution. Dana has two earning ancestors.
- Alice shares 50% of her gross referral commissions; Bob shares 0%.
- Bob has 100 previously issued credits in escrow with Alice; Carol has 200. These are existing principal, separate from the 300 credits being earned in the example.
- Eligibility, escrow balances, and sharing fractions stay unchanged during the example interval. All figures are illustrative, not deployment defaults or promised earning rates.

| Recipient | Core allocation from Dana’s source | Family sharing adjustment | Final earned allocation |
| --- | ---: | ---: | ---: |
| Alice | 30 | −15 | 15 |
| Bob | 30 | +5 | 35 |
| Carol | 0 | +10 | 10 |
| Dana | 240 | 0 | 240 |
| **Total** | **300** | **0** | **300** |

Alice’s 15-credit pool is divided by Bob and Carol’s 100:200 escrow ratio. The escrow principal remains 300. Its equality to the example’s 300-credit attribution is coincidental: escrow is existing inventory, while the attribution is new earned entitlement. The diagram does not spend principal to fund the pool or automatically compound the earnings into it.

## Rules behind the visuals

### Core rewards

For a lane allocation `Y`, member balance `p`, and lane total `P`, a source receives gross attribution `G = Y × p / P`. With `d` earning ancestors and fixed commission `q`, the holder retains `(1 − d × q) × G`; each ancestor receives `q × G`. Each commission uses the original source attribution, rather than a remainder passed up the chain.

The network fixes maximum depth `H` and requires `H × q < 1`, preserving positive own-asset retention at every permitted depth. A member at depth `H` cannot recruit children. The logical protocol root receives no commission; auctioned seats begin at earning depth zero.

The emitter and lane allocator sit outside the referral allocation calculation. The tree divides the budgets it receives. Adding an asset balance, member, or family does not create another global emission stream. A lane with no eligible assets does not store its allocation as backpay for a later entrant.

### Family sharing

A parent can share a fraction `μ` of gross referral commissions `O`. If the immediate children’s total credit escrow is positive, the pool is `μ × O`. A child escrowing `s` out of total escrow `S` receives `μ × O × s / S`.

The pool combines referral commissions from all of that parent’s asset lanes. It excludes own-asset earnings, admission fees, escrow principal, top-seat payments, and incoming family allocations. An incoming family allocation is kept whole and does not feed the recipient’s own family pool.

A child with zero escrow receives no family allocation. If all children have zero escrow, the parent retains its commissions. Children do not need eligible assets or their own recruits to receive family allocations.

The parent controls its sharing fraction for future accrual, including zero. This is a changing offer, not ownership of a permanent percentage of the parent’s future earnings. A new stake does not claim the earlier pool. Earnings are not automatically restaked.

### Admission, movement, and custody

An ordinary join fee transfers issued credits to the parent. The parent can offer a zero fee and can set an increment that raises its quote after each successful join. A joining transaction can enforce a maximum accepted fee. A paid admission fee is not refundable family principal and does not create escrow weight.

A top-level seat requires an authenticated auction award; its final winning credit payment is burned. The specific auction and bootstrap credit inventory belong to the integration. The referral calculation does not mint credits to pay its own first auction.

Family escrow remains the child’s principal, withdrawable after settlement without a mandatory lock, penalty, or forfeiture in the specified model. It does not automatically carry AVA emission-governance weight. The same issued principal cannot simultaneously back family escrow, an AVA pledge, a clearing commitment, or another auction bid.

A node’s parent connection is permanent. Asset balances and family stakes can change under their respective rules. Moving capital to another family requires another node; descendants do not move with that capital, old admission payments are not refunded, and prior earned credits remain claimable through the old node. A free ordinary link can admit a creditless newcomer if such an offer is available; transaction and account costs are separate.

Every change first settles the relevant earlier earnings under the earlier terms. A family escrow or sharing change requires a common settlement cutoff across all lanes funding that family. Collection issues only the member’s net earned entitlement, once.

## Why these rules fit together

Core commissions connect referral ancestry to ongoing eligible holdings. Family sharing lets parents use their referral income to compete for members. A child with family escrow can benefit from commissions generated by sibling branches, so its interest can extend beyond its own downline.

The new descendant’s asset balance does not itself enlarge the parent’s immediate-child credit-escrow denominator. New eligible holdings do, however, change their lane’s asset denominator. Additional child escrow also competes for the family pool. A new holding can contribute to family income without guaranteeing an increase in every existing member’s absolute earnings.

Credit-weighted sharing also avoids an equal allocation for each child account. Splitting a fixed amount of escrow among children under the same parent preserves the owner’s combined ideal family share. This is a specific proportional-accounting property; it does not establish human identity or universal resistance to coalitions. Parents may hold escrow through child accounts they control under the same rules.

A parent’s current sharing percentage alone is not enough to compare offers. The amount of referral income, competing child escrow, admission price, depth, and remaining recruiting room also matter. A smaller pool can provide a larger current share when less escrow competes for it. Current estimates can change as participation, lane allocations, the global schedule, or offers change.

## Scope and source check

These materials explain the intended protocol mechanics and the author’s goal of permissionless operation without operator custody. They distinguish a parent’s own income from a child’s assets and escrow principal. They do not establish the authority or upgrade configuration of a live deployment. The current referral library calculates transitions; its integrating program must authenticate inputs and enforce custody, ownership, and settlement.

Sources reviewed on 16 November 2026:

- [Referral Network](/home/tomtop3/work/mayf/system-codex/docs/github-wiki/content/Referral-Network/Referral-Network.md) — Membership, core commissions, and family sharing.
- [Formal Specification](/home/tomtop3/work/mayf/system-codex/docs/github-wiki/content/Referral-Network/Referral-Network-Formal-Specification.md) — Allocation equations, conservation, custody, and historical settlement.
- [Mutualization](/home/tomtop3/work/mayf/system-codex/docs/github-wiki/content/Referral-Network/Mutualization.md) — Pool funding, child escrow weights, sibling-branch exposure, and withdrawal rules.
- [Link Shopping](/home/tomtop3/work/mayf/system-codex/docs/github-wiki/content/Referral-Network/Link-Shopping.md) — Joining fees, changing offers, and the distinction between permanent links and movable capital.
- [Parameters](/home/tomtop3/work/mayf/system-codex/docs/github-wiki/content/Referral-Network/Referral-Network-Parameters.md) — Fixed depth and commission terms versus participant-controlled offers.
- [Rationale](/home/tomtop3/work/mayf/system-codex/docs/github-wiki/content/Referral-Network/Referral-Network-Rationale.md) — Incentives, proportional dilution, and the limits of splitting-neutrality claims.
- [Participation and Referral Rewards](/home/tomtop3/work/mayf/system-codex/docs/github-wiki/content/Credits/Emissions/Participation-and-Referral-Rewards.md) — Eligible balances, attribution, commissions, and net collection.
- [Emission Accounting](/home/tomtop3/work/mayf/system-codex/docs/github-wiki/content/Credits/Emissions/Emission-Accounting.md) — Preserving old earnings across changing balances and family terms.
- [Referral library README](/home/tomtop3/work/mayf/system-codex/referral/rust/mayf-referral/README.md) — Current implementation responsibilities and its boundary with the integrating program.

The new diagrams and explanation are standalone artifacts. The earlier Mayflower incentive bundle remains separate.
