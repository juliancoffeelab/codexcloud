---
name: deploy
description: Use for an intended live human playtest, or immediately when the user asks to deploy, publish or update the live game.
---

# Deploy

For an intended live human playtest, finish relevant implementation validation
and commit/push required changes first. Follow the
[repository delivery policy](../../../../AGENTS.md).

An explicit deploy/publish/update request enters this workflow immediately.
Use one preview Site throughout the chat. At the first deployment, retain the
first branch name in conversation context as the preview tag; pass it after later branch or
PR changes. If this chat already selected a numeric preview key, retain it instead.
The preview URL follows the chat's latest deployment.

Invoke `node scripts/site-operation.mjs deploy --preview <retained-tag>` through
the exact [operation transport](operations.md#code-mode-invocation).
Only an explicit request for live/main deployment selects plain `deploy` without
`--preview`. The tag selects the destination; current committed `HEAD` selects
the source. Main and preview Sites keep separate release receipts and audiences.

The operation owns source synchronization, cached build stages, packaging,
credentials, exact-release selection, audience and deployment. It pins the source
commit and saved version, reuses an exact release when possible and leaves
staged, unstaged and untracked implementation changes alone. A clean tree,
automatic implementation commit or implementation-repository push is not a
prerequisite. Never substitute an older successful release.

This invocation and its operation-owned tool traffic are the entire attempt.
Apart from selecting the retained preview tag, no agent-side discovery, file inspection,
checks, Git operations, manual builds, separate hosting calls or delegated
preparation precede or accompany it. Add no
test, lint, format, content-check, clean-tree, branch-policy, HTML-reference,
archive-audit or post-deployment browser gates. Compiling and packaging the
requested source are production work, not validation suites.

## Result and recovery

Success is `deployed: true` with `url`: return that literal deployment URL and
stop. A URL alone does not prove the requested source was deployed.

On failure, announce the failed stage and concrete error, the next investigation
or repair and why the update is delayed. Include a known site address only as
**not updated**; never invent one or present the previous version as success.
A failure ends the attempt, not the deployment task.

Inspect and fix only the observed blocker, commit/push required source changes,
and rerun the operation. The retry pins the repaired commit; use its build as
verification instead of adding a general audit, tests or formatting cycle.
The operation already owns exact-release and build-stage reuse, transient retries, deleted-cache
recovery and serialization; do not duplicate these agent-side. No new permission
is needed for focused repairs or retries. Keep the user informed as the diagnosis
changes. Continue until deployment succeeds, the user stops the work, or a
concrete external prerequisite requires user action. Report that blocker and the
known site's unchanged status if blocked.
