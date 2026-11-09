---
name: git-workflow
description: Use when fetching branches, inspecting latest main in another checkout, or pushing amended feature commits.
---

# Git workflow

Inspect `git status --short --branch`, the current branch, and
`git config --get-all remote.origin.fetch` before choosing an operation. Cloud
checkouts can fetch only main, so a general fetch need not populate feature refs.

- Fetch a needed branch explicitly, e.g.
  `git fetch origin refs/heads/main:refs/remotes/origin/main`.
- For latest-main inspection when the starting branch has diverged, preserve the
  checkout and use an ignored `tmp/` worktree at the fetched commit. Run tools from
  that checkout and reuse supplied toolchains; do not reinstall the environment.
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
