# mayf-referral

The Mayflower referral and emissions kernel, as a pure state machine. Credits are
emitted at a known rate, and gauge votes split them among lanes. A member takes a
top-level seat or joins under a parent, so top-level members are siblings with no
root above them. Every lane runs along all the links between members, and those
links divide each lane's credits among them. A Solana program and a desktop client
run it the same way.

The wiki's [Credit Emissions](../../../docs/github-wiki/content/Credits/Emissions/Credit-Emissions.md)
and [Referral Network](../../../docs/github-wiki/content/Referral-Network/Referral-Network.md)
pages specify the rules. The `program` module lists every action, one per file
in `src/program/`. The crate documentation explains the layers, the records, the
actions, and the host's duties:

```bash
cargo doc -p mayf-referral --open
```

The emissions example runs one hour of the wiki's examples, from the first join
to the last collection:

```bash
cargo run -p mayf-referral --example emissions
```
