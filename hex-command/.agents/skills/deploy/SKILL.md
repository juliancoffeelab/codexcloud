---
name: deploy
description: Use for an intended live human playtest, or immediately when the user asks to deploy, publish or update the live game.
---

# Deploy

Commit and push completed requested repository changes by default. Deployment
is only for an explicit request to deploy/update the live game, or a task that
explicitly intends a live build for a human to playtest. Merely editing playable
content, running simulations, or pushing changes does not establish that intent.
Without it, finish with the pushed changes; do not enter this workflow. Honor
local-only requests, unpublished releases, and deployment holds.

For an intended human playtest, complete relevant implementation validation and
commit/push required changes before entering this workflow. Reviews,
documentation, tooling-only work, drafts, renders, and simulations do not trigger
deployment on their own.

An explicit deploy request enters this workflow immediately. Do not add
implementation validation or repeat previous checks before invoking it.

For an explicitly requested independent preview, use
`node scripts/site-operation.mjs deploy --preview`. Plain `deploy` targets the main
Site. Preview configuration and saved releases are separate; never substitute the
main Site when a preview is requested.

Invoke the selected command through the exact
[operation transport](../publish/operations.md#code-mode-invocation).
It resolves the exact committed release internally, reusing its saved version
when available and building only when necessary. It passes that receipt directly
to deployment and waits for the terminal result. Those operations own source synchronization,
necessary builds, Sites source synchronization, packaging, credentials, release
selection and audience. Preparation uses the current committed snapshot (`HEAD`).
It leaves staged, unstaged and untracked changes alone: no clean-tree prerequisite,
automatic implementation commits or implementation-repository pushes. Committing
required game changes belongs to finishing implementation, not to a deploy request.

No deployment-time test, lint, format, content-check, clean-tree, branch-policy,
HTML-reference or archive-audit gates may be added. Compiling and packaging the
requested source are production work; running validation suites is not.
Never substitute the latest saved version or an older successful build. The
operation pins `HEAD`, the built Site commit and its saved version throughout.

This invocation and its operation-owned tool traffic are the entire deployment
attempt. No agent-side discovery, file inspection, checks, Git operations,
manual builds, previews or separate hosting calls precede or accompany that
attempt. Do not delegate them. The recovery rule below applies only after the
operation has actually failed.

Success is `deployed: true` with `url`; return that literal production URL and
stop. A URL alone is not evidence that the requested source was deployed.
On `deployed: false`, return the supplied site address with an explicit statement
that the requested update has not gone live, as a progress update rather than
ending the task. Never present the previous live version as a successful update.
If no address is available, explain the failure without inventing a URL. The operation owns exact-release
reuse, transient retries, deleted-cache recovery and concurrent-run serialization;
do not duplicate those steps agent-side.

A returned failure, including a build or preparation error, authorizes a focused
repair and another deployment attempt. Before the first investigation or repair
tool call, announce the failed stage and concrete error, what you will investigate
or fix next, and why the requested link is delayed. If the cause is not known,
say so rather than guessing. Keep the user informed when the diagnosis or repair
changes. Do not silently switch from deploying into development.

Inspect only what the observed failure requires, fix that blocker, commit/push
required source changes, and rerun the deploy operation. The next attempt pins
the repaired commit; never retry an older receipt or call the previous live
version the repaired deployment. Use the operation's build as verification;
do not add a general audit, tests, lint, formatting cycle or unrelated changes.
No new permission is needed for these repairs or retries. Continue until the
requested version is live, the user stops the work, or a concrete external
prerequisite cannot be resolved with the available access. For that last case,
explain the actual blocker and required user action, include the known site
address clearly marked as not updated, and do not claim successful deployment.
