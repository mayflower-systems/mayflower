# Instruction scaffold

The public instructions mirror `mayf_referral::program`, with `create_network`
as the Solana name for `create_protocol`. The kernel owns accounting; handlers
will authenticate accounts, call the corresponding action, apply custody
effects, and persist the entire receipt atomically.

`create_network` is implemented. Every other handler returns
`InstructionNotImplemented`. Their contexts describe primary records and
signers; they are provisional until source, auction, and token accounts are
added. An error rolls back Anchor's preliminary account initialization too.

| Instruction           | Authorization                    | Kernel action / remaining work                                                                                                                   |
| --------------------- | -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `create_network`      | Authority and rent payer         | Validate immutable terms; create an empty protocol and bind its credit mint.                                                                     |
| `register_lane`       | Network authority and rent payer | Authenticate the current earnings baseline; register and persist a permanent asset slot.                                                         |
| `set_balance`         | Network authority and rent payer | Authenticate current earnings and the complete ancestry; settle old eligibility and store all changed records with the eligible position change. |
| `join_at_top_level`   | Winning owner and rent payer     | Authenticate and consume a scarce-seat auction award once; burn its price and activate the node atomically.                                      |
| `join_under_parent`   | New owner and rent payer         | Enforce the maximum fee; pay the parent's owner, increment the offer and create the permanent child.                                             |
| `set_offer`           | Node owner                       | Replace the ordinary admission price and increment.                                                                                              |
| `set_sharing`         | Node owner                       | Settle every source under old terms, then apply the prospective sharing rate.                                                                    |
| `set_credit_stake`    | Child owner                      | Settle every parent source and old gifts; transfer the principal delta into or out of exclusive escrow.                                          |
| `collect_credits`     | Node owner                       | Mint already-accrued net whole credits and retain the fractional claim.                                                                          |
| `settle_node_in_lane` | Permissionless                   | Settle an existing lane position and complete ancestry against authenticated earnings.                                                           |
| `settle_family`       | Permissionless                   | Settle every parent source and credit the child's incoming gifts.                                                                                |

Rent payers can differ from owners and authorities. A settlement caller does
not need the owner's signature. Ordinary joins do not need a parent's signature;
the posted offer supplies the terms.

## Account identity and storage

- Network: `["network", authority]`, one network per authority.
- Lane: `["lane", network, asset]`; the kernel assigns consecutive lane indexes.
- Node: `["node", network, owner, node_index_le_u32]`, allowing several nodes
  per owner. Parent links and ownership are fixed in this scaffold.
- Lane position: `["lane_node", lane, node]`; both records belong to the same
  network, and the position also stores that network explicitly.

The account layouts retain every kernel accounting field. Whole balances and
fees use `u64`; rates use raw micro-basis points; fractional accounting uses
32-byte little-endian `U192F64` storage. Carry fields keep their kernel numerator
units. State conversion must preserve every field exactly. The lane mask and
materialized lane positions survive complete withdrawal.

## Wiring still required

The earnings adapter must authenticate the source bound to each lane, prevent
assigning the same budget twice, and prove that its total is current through an
eligibility change. A monotonic total or
the previously consumed lane cursor alone cannot prove freshness. Public
settlement arguments therefore do not accept arbitrary cumulative earnings.

Define a bounded remaining-account layout for parent-first ancestry, canonical
lane positions, and complete family source sets. Validate program ownership,
network membership, PDA identity, account ordering, writability, duplicates and
the permanent lane mask before converting records. First balance materialization
must allocate every required position; existing-position settlement must reject
missing historical records. Decode paths into heap storage, and measure the
chosen depth against Solana execution limits.

The auction adapter must bind an award to the network, winner and node and
enforce scarcity and single use. There is no free top-level admission or local
auction-issuance instruction in this scaffold.

Token wiring must bind the network credit mint, payout destinations, burn
authority, mint authority and exclusive escrow. Network creation records an
existing non-native SPL Token mint; it does not acquire mint authority or create
bootstrap credits. Add custody accounts and CPIs before enabling the affected
handlers. Apply all receipt records and custody effects in one transaction.

The emitter, auction format, custody authority model and deployment address
remain separate integration work. The kernel's
[host contract](../../../../../rust/mayf-referral/src/lib.rs) and
[economic specification](../../../../../rust/mayf-referral/SPEC.md) define the
requirements those integrations must satisfy.
