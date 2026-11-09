# GitHub wiki

`content/` holds the pages of the
[Mayflower wiki](https://github.com/mayflower-systems/mayflower/wiki). It is a
git subtree of the wiki's own repo,
`https://github.com/mayflower-systems/mayflower.wiki.git`. This README sits
outside `content/`, so it never reaches the wiki.

A subtree is a copy. The pages are ordinary files in this repo, so every clone
has them, and you edit and commit them like any other file. Git does not keep
the copy and the wiki in sync. You do that with two commands:

- `git subtree pull` brings wiki changes into this repo.
- `git subtree push` sends this repo's changes to the wiki.

The Wiki check in CI fails whenever `content/` differs from the wiki.

## Setup

Add a remote for the wiki. Remotes live in each clone's own config, so do this
once in every clone:

    git remote add wiki https://github.com/mayflower-systems/mayflower.wiki.git

Run every `git subtree` command from the repo root. `git subtree add` and
`git subtree pull` also need a clean working tree.

## Bring wiki edits into this repo

People can still edit pages on GitHub's website. When they do, the Wiki check
fails on every pull request until this repo catches up. To catch up, start a
branch from an up-to-date `master`:

    git switch master && git pull
    git switch -c pull-wiki
    git subtree pull --prefix=docs/github-wiki/content wiki master --squash

This adds two commits. The first squashes the wiki changes into one commit. The
second merges that commit into `content/`. Open a pull request, and merge it
with a merge commit.

## Send your page edits to the wiki

Edit pages under `content/` on your branch and commit them. Before you merge,
send them to the wiki:

    git fetch wiki
    git subtree push --prefix=docs/github-wiki/content wiki master

The Wiki check on your pull request fails until you do. `git subtree push` sends
only the changes under `content/`, so commits that also change code are fine. It
needs the wiki's commits in your clone. Without the fetch, it fails with
`could not rev-parse split hash`.

If the push is rejected, the wiki has changes that your branch lacks. Run the
`git subtree pull` command above on your branch, then push again.

## Rules

- **Merge pull requests that change `content/` with a merge commit.** Do not
  squash or rebase them. Those merges can break the record that `git subtree`
  keeps in commit messages, and the next `git subtree pull` then deletes most of
  this repo. To rule this out, allow only merge commits in the repo's Settings →
  General → Pull Requests.
- **Always pass `--squash` to `git subtree pull`.** The subtree was added with
  `--squash`. Without it, git refuses to merge.
- **Link to pages by name.** Links use wiki page names, not file paths.
  `[floor raising](AVM-Floor-Raising)` points to `AVM-Floor-Raising.md`,
  wherever it sits under `content/`. These links work on the wiki, but not in
  GitHub's file viewer.
- **Keep page file names unique** across all folders. The wiki finds a page by
  its file name alone.
- **Keep the published wiki self-contained.** Pages under `content/` link to
  other wiki pages, not source files, READMEs, or specifications elsewhere in
  the repository. Include the definitions needed to understand each algorithm in
  the wiki itself. Repository modules may link into the wiki.
- **Separate specification from rationale.** Specifications define state, units,
  equations, valid transitions, rounding, and settlement. Rationale pages
  explain incentives and tradeoffs without adding rules or overstating
  guarantees. Distinguish exact mathematical identities from finite-precision
  behavior and proposed features from supported operations.
- **Use a consistent page structure.** Keep title, summary, and tags in the
  opening metadata comment; use sentence-case section headings and page-name
  links. Lead overview pages with purpose and navigation rather than accounting
  details.
- **Describe an open-source protocol.** Identify participants, state, configured
  roles, and custody rules rather than implying a company or managed service.
  Distinguish permissionless public actions from ownership and configuration
  authorization. Keep paid admission, commissions, and other economic effects
  explicit; terminology is not a substitute for describing the actual mechanism.

## The CI check

`.github/workflows/wiki.yml` runs on every pull request, on every push to
`master`, and by hand. It compares `content/` with the wiki's latest pages. When
they differ, it fails and lists the files that differ. It does not say which
side changed. Pull wiki edits in, push your page edits out, or both, until the
two match.

## Repair

Use this if a pull request with `git subtree add` or `git subtree pull` commits
was squash- or rebase-merged. Do not run `git subtree pull` first. This replaces
`content/` with the wiki's pages, so first make sure the wiki has every page
change you want to keep. Then:

    git switch master && git pull
    git switch -c repair-wiki
    git rm -r -q docs/github-wiki/content
    git commit -m "Remove docs/github-wiki/content to repair the subtree"
    git subtree add --prefix=docs/github-wiki/content wiki master --squash

Open a pull request, and merge it with a merge commit.
