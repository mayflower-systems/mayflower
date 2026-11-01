---
title: Protocol Parameters
summary:
  How maximum depth, the override rate, and each member's voluntary sharing
  fraction shape the referral allocation.
tags:
  - referral
  - parameters
---

The [[referral-network|Referral Network]] has two fixed network parameters:
maximum depth $m$ and override rate $q$. Node owners separately choose
voluntary sharing and admission-offer terms.

## Maximum depth

Depth counts the participating links from a top member: top members are at depth
0, their children at depth 1, and so on. The maximum $m$ is the greatest
permitted depth. A source at depth $d$ has $d$ members in its upline, each
eligible for an override from that source.

The reference $m=2$ permits three participating levels: depths 0, 1, and 2. A
larger maximum depth permits longer recruitment chains; a smaller maximum limits
the number of upline members receiving overrides. With $m=0$, only top
positions participate and there are no overrides.

## Override rate

Every source assigns fraction $q$ of its original own points to **each**
upline member and retains $1-dq$ at depth $d$. The fixed parameters satisfy

$$0\le q<1,\qquad mq<1.$$

The strict product bound keeps the own-point retention of every permitted member
positive. The reference $q=10\%$ with $m=2$ gives:

| Depth |                     Overrides to upline | Own-point retention |
| ----- | --------------------------------------: | ------------------: |
| 0     |                                      0% |                100% |
| 1     |                                     10% |                 90% |
| 2     | 20% in total, 10% to each upline member |                 80% |

Increasing $q$ shifts more of each downline member's core attribution to its
upline. Setting $q=0$ disables overrides. The network fixes $m$ and $q$
when it is established; the reference values illustrate a configuration rather
than an automatic default.

## Voluntary sharing

Each node owner chooses a sharing fraction $\mu$ from 0% to 100% for each
channel, initially 0%. It applies to that member's core earnings plus incoming
gifts, divided among direct children in proportion to their own points. If those
children have no own points, the member retains the full amount. A member's
sharing policy can change for future earnings; the previous policy determines
the attribution of earlier earnings.

Overrides and sharing play different roles. An override sends a fixed fraction
of every source's own attribution upward to each upline member. Mutualization sends a
member's chosen fraction of available earnings downward to its direct children.
Both redistribute the same underlying participation or reward; neither creates a
new budget.

## Joining offers

A node owner can offer positions beneath its node, subject to the network's
maximum depth. The offer sets the current joining fee and the increment added to
that fee after each successful join. The owner may change those values, close
the offer, or replace it; changes affect future joins and leave existing
uplines intact. [[referral-network--link-shopping|Link Shopping]] explains how
admission terms and sharing affect a prospective member's choice.

The network's maximum depth and override rate are fixed for its lifetime. A node
owner's joining offer and voluntary sharing policy do not change those network
parameters.

## Time and rounding

A member's points and sharing policy can change while the reward rate changes.
Attribution uses the values in effect during each interval. Sharing is rounded
conservatively to the accounting grid, with amounts too small to divide retained
by the sharing member. The
[[referral-network--formal-spec|formal specification]] gives the calculations;
[[referral-network--mutualization|Mutualization]] illustrates them.
