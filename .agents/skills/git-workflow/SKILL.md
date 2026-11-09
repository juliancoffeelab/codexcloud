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
- For a fresh task on latest main, use the existing checkout: fetch main explicitly,
  then `git switch main` and `git merge --ff-only origin/main`. If main does not
  exist locally, use `git switch -c main --track origin/main`. Preserve uncommitted
  work and local commits. If a known upstream force-push prevents fast-forwarding
  and the checkout is clean, preserve the old local main with
  `git branch -m main archive/main-before-rewrite-UNIQUE` (choose an unused name),
  then `git switch -c main --track origin/main`. This replaces the stale local
  baseline while retaining its commits; it does not merge the unrelated histories.
  For unexplained divergence or uncommitted work that blocks switching, report
  the blocker. Create implementation branches from fetched main as required by
  AGENTS.md.
- Use a separate worktree when the task actually needs simultaneous checkouts or
  preserving active work prevents switching. Reuse supplied toolchains and caches.
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
