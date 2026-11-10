---
date: "2026-11-09"
slug: "path-to-permanence"
---

# Path to permanence

## Summary

- The Mayflower protocol is going open-source
- The open, permissionless, non-custodial protocol will be accessible through open-source software clients
- Documentation, discussion, announcements, and clients will be hosted at: [https\://github.com/mayflower-systems/mayflower](https://github.com/mayflower-systems/mayflower)
- The migration of Solana program state from the current Nirvana/Samsara implementation into the open-source Mayflower platform will be a one-time upgrade in order to enshrine the full specification of the protocol in immutable blockchain-hosted executables

## Why “blockchains” at all?

The “cryptocurrency” experiment comes down to one problem: can a system stay operational regardless of who engages with it? Take Bitcoin for example. For a piece of software, its history shows far fewer emergency administrative interventions than would be expected. I wouldn’t call the Bitcoin system truly “decentralized” \- but it gets awfully close to that ideal. Some fifteen years after its own creator vanished from the face of Earth, the ledger continues chugging along with a resilient life of its own.

I doubt the word “blockchain” would ever have broken into mass media were it not for the myth of the absconded founding father. What better proof of “unstoppably decentralized” could you hope for? By my lights, Bitcoin sets the gold standard.

When I say Bitcoin sets the “gold standard,” I mean this figuratively. A blockchain program should be like gold itself: chemically inert, practically indestructible, functionally immortal. In practice, this means absolute zero custodial administration. That’s the north star any self-described blockchain project should aim for.

The light of that north star is getting dangerously faint. People have ideas for protocols. But then that idea materializes into a product, the product attracts attention, the attention gathers capital, and the capital grows a thick crust called a “company.” Crusts are hard, but not strong, and even less ductile. Companies are typically volatile & fragile things. The irony is that they make a poor home for something meant to be permanent. I think it’s fair to suggest that the rampant company-ification of the blockchain space around 2020 is how it got lost, and why it still feels listless today.

I’m not exempt. My original goal was to host a simple mathematical algorithm on a blockchain, so that anyone can do anything with it for as long as they want. Little by little, this goal got stymied by wasteful misadventures in company-isms and time-draining distractions from trying to engage with a culture I don’t share. I began to forget the joy that comes from turning ideas into programs that run like perpetual motion machines. The beautiful, bizarre opportunity with blockchain platforms is that anyone can release pure programs into the wild, and let them take on lives of their own.

So, enough of that. Back to basics, with a more principled approach.

The “Nirvana” protocol (and its “Samsara” spin-off) were fine first prototypes of what I like to call an “Amplifying Value Machine.” The AVM is a specification for a system of equations that produces a non-decreasing exchange rate along two axes (see https\://github.com/mayflower-systems/mayflower/wiki/Amplifying-Value-Machine). I am always happy to see that -- despite all the churn in the world of cryptocurrencies -- the AVM protocol keeps on doing what it is supposed to do: algorithmically offering redemptions and issuance according to a deterministic rate schedule, and permitting anyone to raise the minimum redemption rate according to algorithmic invariants.

But its economic machinery is incomplete. I see three missing elements:

1. Easy openness to network effects
2. Sustainable incentive to participate in the network
3. Absolute zero custodial involvement in incentive settlement

The “Mayflower” project is an upgrade of the essential ideas, with key improvements on its sustainability & scalability. With lessons learned, it is set up to grow on its own.

After all, “decentralization” is a high bar to clear:

- People are free to participate, contribute, and create
- Incentive does not depend on a central entity, but on the system itself
- No individual person can cause the system to break its rules

The curious thing about “blockchains” and “cryptocurrency” is that their “decentralization” makes contact with the material world. People are mining BTC with electricity they expect to pay for with the BTC credit the software gives them. I suspect most “DeFi” products forget that you must decentralize the “participation” and the “incentive” also – or else the self-anointed “DeFi” label is just a wishful tactic to launder accountability for a centralized team of custodians who would prefer to not be seen as such.

For years I’ve been thinking about how to finish the job of decentralization. Here is how this project will finally move all its components toward the ideals of “cryptocurrency.”

## Free and open source

The core code will be open source and free to use. I am in the midst of copying my work into a public GitHub repository [https\://github.com/mayflower-systems/mayflower](https://github.com/mayflower-systems/mayflower). Development will be out in the open. Not only the implementation, but its specification, theory, public discussion, and public announcements.

The protocol has always been open, and its key components source-available (via development kits, interfaces, test suites). But I was not loud & proud about sharing the code. This reticence was mainly due to a serious concern for security. I have grown used to waking up at 3am to re-re-review the code, just in case an exploitable defect slipped through all the audits, fuzz tests, and formal proofs. My second reason for code secrecy was a caveman-brained instinct to “protect” the value of the product from being cloned into oblivion. Whether or not these were sound reasons at the time, the rise of AI makes them decisively irrelevant at this point. In this brave new world, “obscurity” is neither “security” nor a “moat”. So we might as well bite that bullet.

With the way AI raises the stakes for security, the rate of writing mission-critical code has gotten (ironically, for me) much slower. With open-source, anyone can find any chink in the armor. I find my “serious concern” about security resembling good-old “paranoia.”

On a hopeful note: going open source is also positive leverage. People will find it much easier to create their own clients, integrations, and mutant spin-offs of the ideas in the protocol. I hope they do. I hope they can point an LLM at the codebase and go to town. The network effect from being freely out in the open is a mindshare-multiplier I am certain I am guilty of underestimating.

## Referral network and revenue share

The improvement I am most excited about is a sound and scalable method for incentivizing participation in Mayflower. You can find a lengthy specification in the [https\://github.com/mayflower-systems/mayflower/wiki](https://github.com/mayflower-systems/mayflower/wiki). Here is a short summary that focuses on how this system aligns with the mission for long-term self-sufficiency.

Designing a sound referral system for blockchain environments has taken much longer than expected. It’s a thorny problem where the cost of spam, sniping, and cloned identities is near zero. I am yet to find an example in the space that is not trivially self-defeating from self-referring. If you capture value from your recruits, you might as well recruit yourself and pocket a discount that benefits no one else. Moreover, most (all?) referral systems fail to provide a reason for a newcomer to choose one referral code over another. Referral systems are supposed to grow the product \- but they tend toward a net-neutral homeostasis giving an illusion of growth, while adding nothing.

Mayflower’s referral & reward share system is similar in spirit to how blockchains themselves incentivize honest transaction processing. And I bet this feature of Mayflower will be what ends up mattering the most.

First: where does “revenue” come from? Anything other than organic fees paid in real assets by self-interested participants for a service does not count as such. Mayflower does not print fake money and pass it off as “revenue.” The revenue the protocol gathers comes purely from fees on transactions that willing participants elect to pay out of their own individual pockets.

In the Mayflower protocol, “revenue” is the word for quantities of 3rd-party tokens collected and disbursed algorithmically. As a pure algorithm, the protocol makes no claims that the things it settles are anything other than what they are: quantities of fungible tokens managed by 3rd-party programs. No privileged entity will have custody of this “revenue.” No authorized operator has the dangerous task of managing it. The revenue clearing is completely transparent and settled automatically out in the open. Keeping discretionary asset management out of the protocol is meant to reduce reliance on custodians and the security risks that come with them.

Every instance of an “Amplifying Value Machine” created in the Mayflower program funnels revenue into a non-custodial, open-source, rules-based distribution program.

Since the program requires zero entanglements with counter-parties and custodial men-in-the-middle, the streams of revenue from each market get escrowed plainly in the exact token the AVM is based on. There is no authorized desk that swaps the various tokens into a common currency (for example a digital dollar like USDC or a blockchain-native token like Solana’s SOL). Fewer moving parts means fewer broken pieces.

The problem of “distribution” is where things get interesting. Any attempt to measure the value of these various pools of escrowed tokens by a common unit would also be custodial. You cannot ascribe a dollar value to tokens without trusting someone to tell you what it is. Consequently, the protocol only sees pots of digital tokens X, Y, Z and has no opinion on what each token is worth.

Without an opinion on the “real-world” value, Mayflower clears the revenue by auctioning it off. But an auction requires a common unit for the bids. What should agents bid with? It would be absurd to bid on auctioned Bitcoin with USDC (or the reverse). This key piece of the puzzle is handled by Mayflower native “credits.” Participants spend their credits to bid on the auctioned-off lots of revenue. And so rather than handling revenue share via central planning, participants compete over revenue in a decentralized open auction. As a side effect, the auction calibrates the value of these credits.

(Note: Currently, the “prANA” token functions similarly to this in the Nirvana & Samsara clients. When the migration to Mayflower happens, prANA will be converted into these “credits” 1:1 and continue functioning as the protocol-native way to access protocol-generated revenue. The accumulated prANA tokens are a major carry-over from the current clients to the new one – prANA functions as a record of skin-in-the-game)

Credits are algorithmically created, and designed to be scarce. They are not given away by central discretion, but earned by those who deserve them most. Here is where the economic network effects for Mayflower take flight: credits are earned only through a referral system. A referrer earns credits in proportion to the aggregate of their referees.

An account earns credits proportional to its skin-in-the-game, the sum of its referees’ skin-in-the-game, and the sum of its referees’ referees, recursively all the way down. The transitive weight of referee earnings is worth lingering on.

The referral system is a network where rewards accrue upwards. When Alice refers Bob to use Mayflower, then Alice earns credits proportional to her holdings plus Bob’s holdings. When Bob refers Elon Musk, Bob gets his big bonus – and so does Alice\! Alice gets a bonus from Bob having referred Musk. Even if Alice is not herself attracting lucrative referees, she still benefits if someone has her for an ancestor.

All together, this is an algorithmic, transparent, and fair “multi-level marketing” structure. The incentive is supplied from real revenue. Credits are earned through participation in the referral network, even without recruiting anyone. And the benefits of recruiting other people do not stop at your reach, but branch out in a multi-level tree that feeds upwards to your account.

There is much more to say about it. Now that things are getting open-sourced, the curious can read the code (or direct their AI agent to interrogate it).

## Self-reliance

With these improvements, the goal is for the protocol to run as an open public good for as long as Solana keeps settling transactions. I am open-sourcing client implementations that only need a connection to a Solana RPC node in order to run without any reliance on a centralized infrastructure. The aim is for the protocol to keep running even if a particular frontend or its operator can no longer do so. That’s what “protocol” means to me.

Mayflower’s referrals & revenue module is just a protocol: self-contained, self-consistent, self-reliant. Which is to say “non-custodial.” The protocol collects revenue from everything that uses it, and shares that revenue algorithmically in proportion to how much skin in the game participants and their referees put in.

I can understand why other “DeFi” companies take custody of their revenue and manage a “treasury.” But this behavior yokes the product to counterparties who can fail at any moment, and depends on privileged humans showing up to their desk to make the right decisions. There is nothing inherently wrong with that style, but there is also nothing “decentralized” about it. I am sure the “user experience” can benefit from a central custodian gathering up tokens, liquidating them through trusted counterparties, spending the cash with day-to-day discretion, and delivering dividend payments in targeted campaigns. But self-reliance means autonomy. And only immutable, pure algorithms can hit that target.

## Layer 2

The core protocol will be an open public good. Its open-source software clients will be provided for free. This is the only way to ensure the protocol and participation in it depend on no one in particular. My main attention is on getting this right, so it is stable and enduring.

But the larger endeavor is a two-sided operation: the free & open core, and a for-profit business that layers on premium experiences. For a good example of this symbiosis, consider WordPress & Automattic or Kafka & Confluent.

To this end, I have also been building out a set of “gateway” applications. They layer on a premium experience when interacting with the core Mayflower platform. “Mayflower Reserve” will be the flagship application which hosts a set of simple (and boring) AVM instances. The “Mayflower Reserve” app will provide a frontend for the USDC reserve (the AVM parameters currently called “Nirvana”) and the various AVMs that are listed in the “Samsara” client.

For those building on top of the core Mayflower protocol, there will be a purpose-built dashboard called “Shipyard.” This product streamlines and extends the features of the core protocol for the sake of building _with_ Mayflower’s core engine. And adds features like telemetry and analytics, as a service provider should.

Since “Reserve” and “Shipyard” are both layered on top of the core protocol, the core protocol collects and shares out the “revenue” from them all. But since these gateway applications have recurring costs for infrastructure and application development, they take a “frontend fee” in order to pay for the 3rd party services that keep them going. Their business model is conventional. But the core protocol is emphatically unconventional: decentralized, with open-source clients anyone can run.

The reason for the explicit separation between the for-profit businesses & the protocol is so the work that goes into the Mayflower platform can be less exposed to the bad incentives and vulnerabilities that come with company-isms. A bright line between these two layers should have been there all along, and would have hastened its roll-out. Not only is this two-level symbiosis a proven successful model, but it cleanly isolates the core protocol as thoroughly non-custodial. Realistically, a system cannot aspire to permanence if its survival depends on a particular company or the laws the regulate companies. To achieve “decentralization” you have to make it so that no one in particular matters more than anyone else – this goes for core creators, as well as for government bureaus.

## Next

This upcoming upgrade has been long in the making. To squeeze out every last drop of potential “custody” is not a typical concern for a computer programmer. But a design for closing the loop on how the protocol collects and distributes revenue to accounts that meaningfully contribute skin-in-the-game to the system is a substantial step toward genuine decentralization. Although the current program state (Nirvana/Samsara) continues to implement the core protocol, the scalability of the participation in the program could be greatly ehnanced by the new decentralized incentive modules. And access to the blockchain programs will be aided by open-sourcing the clients.
