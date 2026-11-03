# mayf-referral

`mayf-referral` is the concrete Mayflower referral program built on the generic
`referral-tree` algorithms. It keeps the membership records of a referral
network: who joined under whom, which offer each member joined through, and how much each
member has deposited in each scope. It is the identity layer for the
`referral-tree` crate. That crate computes points from anonymous numbers and
cannot tell which deposit belongs to which member. This crate supplies those
relationships and checks them.

The crate is pure `no_std` Rust. Identifiers have a generic type `I`, such as
an account address. Numeric representations are fixed by this program. The
crate has no storage, signature checks, token transfers, emissions, or reward
math.

The host application does the rest. It authenticates the caller, loads the
records, and calls an action. It then stores every returned record and executes
any fee transfer, all in one atomic step. Actions take records by value and
return new ones, so an error leaves stored state unchanged.

## Numeric policy

`referral-tree` defines algorithms over `FixedNumber`. This program chooses the
implementations from `mayf-fixed-drivers`; its callers supply no numeric type
parameter.

| Quantity                                                 | Representation      | Reason                                                                         |
| -------------------------------------------------------- | ------------------- | ------------------------------------------------------------------------------ |
| Token amounts, fees, and current direct/subtree balances | `u64`               | Matches token base units and bounds the scope's total deposits                 |
| Clock ticks                                              | `u64`               | Matches the referral algorithms' lifetime bound                                |
| Lifetime deposit/withdrawal counters                     | Checked `u128`      | Counts repeated flows, which can exceed a current token balance                |
| `ReferralRate`                                           | `U64F64`, 16 bytes  | Compact unsigned override and sharing fractions with 64 fractional bits        |
| `ReferralPoints`                                         | `U192F64`, 32 bytes | Points, cumulative point-time, and sharing indexes on the same fractional grid |

A full `u64` scope balance over a full `u64` lifetime needs up to 128 integral
bits. Preserving 64 fractional bits therefore needs at least 192 raw bits.
The existing `U192F64` driver provides 256-bit storage and checked wide
intermediates. It covers that full bound without restricting valid token amounts
or lifetimes. This is a capacity choice, not a target-specific performance claim.

`ReferralRate` and `ReferralPoints` both have quantum `2^-64`. Convert a rate
with `ReferralPoints::from(rate)`; widening is exact and never rounds it again.
The program exposes `ReferralTreeConfig` and `ReferralTreeNode` as the concrete
specializations used for referral arithmetic. Numeric implementations, grids,
and encodings are program choices; the underlying limb algorithms remain in
the drivers crate.

## Network and root

`create_network` creates the network and its administrative root node. The
network fixes two values for its whole life:

- **`max_depth`**, the deepest participating level.
- **The override rate** `override_numerator / override_denominator`: the
  fraction of each deposit that `referral-tree` credits to each upline member.

The crate converts the rate to `ReferralRate` (`U64F64`), rounding up. Upline
members then never get less than the stated rate, and a small positive rate
cannot round to zero.
`ReferralNetwork::override_rate` returns this compact value.
`ReferralNetwork::config` widens it exactly and returns `ReferralTreeConfig`,
which supports both point calculations and full-lifetime point-time settlement.

## Nodes, paths, and depth

Every member is a permanent node. No action moves or removes a node.

A node stores its whole path to the root. Index 0 is the node itself, and the
last populated index is the root. `depth` counts the nodes in that path. The
root has depth 1, members who join the root have depth 2, and so on.

A path holds at most `max_depth + 2` nodes: the root plus `max_depth + 1`
participating levels. `MAX_PATH_NODES` is 8, so `max_depth` is at most 6.

## Offers

A node admits direct children through its active offer. It has at most one
active offer at a time, and nodes at the deepest level cannot publish one.

- **Publish.** `publish_offer` opens an offer with version
  `latest_offer_version + 1`. Versions identify offer generations.
- **Price.** An offer has a `join_fee` and a `join_fee_increment`. Each
  successful `join` charges the current fee, then raises it by the increment. A
  fee of 10 with increment 2 charges 10, 12, then 14.
- **Fee limit.** The joiner passes `max_join_fee`. If the price changes between
  quote and join, the join fails instead of overcharging.
- **Payment.** The fee goes to the parent node's owner. `join` returns it as a
  `FeeTransfer` for the host to execute.
- **Changes.** `update_offer_pricing` changes the fee or increment and keeps the
  version. `close_offer` closes the offer. `replace_offer` closes it and opens
  a new one with a different id and the next version.

Offer changes never affect members who already joined.

## Admission history

On `join`, the new node records the offer id and version for its own edge. It
then copies its parent's edge history. Edge `i` joins path nodes `i` and
`i + 1`. The parent's `joined_children` counts its successful direct joins.

These fields provide provenance and observability. Reward calculations do not
use them. They also have limits:

- Snapshot validation checks only that each edge has an id and a nonzero
  version. It does not prove that those offers existed.
- Repricing does not change an offer's version, so these fields do not record
  the fee paid.
- `joined_children` is a count, not a list of children.

## Lanes

A lane holds one node's deposits in one scope. A scope is a domain of
comparable units, such as one emission channel. Balances from different scopes
never mix. A lane stores:

- **`direct_balance`**: units the node's owner has deposited and not withdrawn.
- **`subtree_balance`**: `direct_balance` plus the direct balances of the
  node's whole downline in the same scope.
- **`lifetime_deposited` and `lifetime_withdrawn`**: the owner's own totals.
  Upline lanes see downline activity only through `subtree_balance`, so one
  deposit is not counted once per level.

`deposit_into_lane` and `withdraw_from_lane` take the full source-to-root lane
path. They change the source's `direct_balance` and the `subtree_balance` of
every lane on the path. Only the node's owner may act. A withdrawal cannot
exceed the source's own `direct_balance`, so an upline member cannot withdraw
a downline member's units. `create_lane` does not check for an existing lane, so the
host must allow at most one lane per node and scope.

The root lane's `subtree_balance` is the scope's total deposits. That total
comes from propagating every source change through its path. It does not need a
direct root deposit.

This crate allows a direct deposit into the root lane. Mayflower Reserve is
stricter. In Reserve, the root sits outside the participating tree. Reserve
rejects point changes on the root and requires the root's direct balance and
reward state to be zero. The root's children are top members at reward depth 0.

## Snapshots and validation

Each record has two forms:

- **The `*Snapshot` struct** has public fields. It is the persistence schema.
- **The validated type** has private fields and read-only getters.

`TryFrom<Snapshot>` checks a record's internal consistency, and `From` converts
it back for storage. Actions accept only validated types, so they never see a
malformed record.

Snapshot validation checks one record at a time. It cannot prove that the
records the host loaded belong together. Two functions check relationships:

- `validate_node_network` checks that a node belongs to the network and that
  its path ends at the network's root.
- `validate_lane_path` checks that lanes form a node's exact source-to-root path
  in one scope.

Every action that takes existing records checks their relationships this way.

## Composing with referral-tree

A lane does not hold everything `ReferralTreeNode` needs. The direct-child
deposit total, the sharing fraction, and the point-time history live elsewhere.
The composing host keeps that state and maps it, together with lane balances,
into `ReferralTreeNode`. The numeric representations are already selected by
this program. Pass `network.config()` to the generic algorithms; type inference
selects `ReferralPoints` without a caller choosing a driver.

The two crates follow different conventions. Depth here counts the root as 1,
while `referral-tree` depth starts at 0 for top members. Paths here run
source-to-root, while `referral-tree` paths run top-to-source.

## Example

```rust
use mayf_referral::*;

let created = create_network(CreateNetworkArgs {
    network_id: 1u64,
    authority: 2,
    node_id: 10,
    owner: 110,
    max_depth: 2,
    override_numerator: 1,
    override_denominator: 10,
})
.unwrap();
let network = created.network;

// The root offers membership at 5 units, rising by 1 after each join.
let published = publish_offer(PublishOfferArgs {
    network,
    parent_node: created.node,
    offer_id: 20,
    join_fee: 5,
    join_fee_increment: 1,
})
.unwrap();

// Owner 111 joins below the root and pays its owner, 110.
let joined = join(&JoinArgs {
    max_join_fee: 5,
    network,
    parent_node: published.next_parent_node,
    offer: published.offer,
    node_id: 11,
    owner: 111,
})
.unwrap();
assert_eq!(joined.node.depth(), 2);
assert_eq!(joined.join_fee_transfer.recipient, 110);
assert_eq!(joined.next_offer.join_fee(), 6);

// The new member deposits 100 in scope 7. The lane path runs source first,
// and unused entries stay `None`.
let scope_id = 7;
let lane = |node: ReferralNode<u64>, lane_id: u64| {
    create_lane(CreateLaneArgs { network, node, lane_id, scope_id }).unwrap().lane
};
let mut lanes = [None; MAX_PATH_NODES];
lanes[0] = Some(lane(joined.node, 30));
lanes[1] = Some(lane(joined.next_parent_node, 31));
let deposited = deposit_into_lane(DepositIntoLaneArgs {
    network,
    node: joined.node,
    scope_id,
    signer: 111,
    lanes,
    amount: 100,
})
.unwrap();
let [member, root, ..] = deposited.next_lanes;
assert_eq!(member.unwrap().direct_balance(), 100);
assert_eq!(root.unwrap().subtree_balance(), 100);

// The root's child is a top member at point depth 0. The host associates this
// concrete numeric state with its member lane; the administrative root is omitted.
let points = ReferralTreeNode {
    deposit: member.unwrap().direct_balance(),
    ..ReferralTreeNode::default()
};
let settled = referral_tree::checkpoint(points, referral_tree::CheckpointArgs {
    config: network.config(),
    depth: 0,
    time: 10,
    parent_point_time_index: ReferralPoints::ZERO,
}).unwrap().next_node;
assert_eq!(settled.point_time, ReferralPoints::from(1000_u64));
```

## Verification

```sh
cargo test -p mayf-referral
```
