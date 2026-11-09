---
name: git-workflow
description: Use when fetching branches, inspecting latest main in another checkout, or pushing amended feature commits.
---

# Git workflow

Inspect `git status --short --branch`, the current branch, and
`git config --get-all remote.origin.fetch` before choosing an operation. Cloud
checkouts can fetch only main, so a general fetch need not populate feature refs.

- Fetch a needed branch explicitly, e.g.
  `git fetch origin +refs/heads/main:refs/remotes/origin/main`.
  The leading `+` lets the remote-tracking ref follow upstream history rewrites;
  it does not reset the local main branch.
- At the start of a fresh task, with a clean working tree, fetch main as above and
  run `git switch -C main origin/main`. The cloud starting checkout can contain
  obsolete history after an upstream force-push; replace that local baseline
  directly. No archive branch, merge or separate worktree is needed. Create an
  implementation branch from this main when making changes, as AGENTS.md requires.
  If the working tree contains uncommitted changes, report them before discarding.
- Use a separate worktree only when the task needs simultaneous checkouts.
- Continue requested implementation on its current branch/PR unless instructed
  otherwise. Divergence alone is not a reason to pull, rebase or replace it.
- For a new feature branch, push with `git push -u origin HEAD`.
- When authorized to rewrite your own feature branch, fetch its remote head,
  inspect it, and use an explicit lease:
  `git push --force-with-lease=refs/heads/BRANCH:EXPECTED_SHA origin HEAD:refs/heads/BRANCH`.
  Stop if the lease fails and inspect the remote changes. A missing tracking ref
  does not justify an unrestricted force push.

Follow the root AGENTS.md conflict and merge authorization rules. Leave other
people's working files and branches intact.
