# CLAUDE.md

## Design docs

`docs/github-wiki/content/` holds the Mayflower design docs: the AVM, credits
and emissions, the referral network, and revenue clearing. Read the relevant
pages before you change protocol logic. Start at `Home.md` in that folder.
`_Sidebar.md` lists every page.

Links between pages use wiki page names, not file paths. The link
`[x](AVM-Floor-Raising)` points to the file `AVM-Floor-Raising.md`, wherever it
sits under `content/`. Page names are unique, so a file search finds the page.

The folder is a git subtree of the GitHub wiki. `docs/github-wiki/README.md`
explains how to sync it. `git subtree push` publishes to the live wiki, so run
it only when asked.
