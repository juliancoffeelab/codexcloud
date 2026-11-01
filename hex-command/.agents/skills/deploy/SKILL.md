---
name: deploy
description: Use when the user asks to deploy, publish or update the live game.
---

# Deploy

Invoke `node scripts/site-operation.mjs deploy` through the exact
[operation transport](../publish/operations.md#code-mode-invocation).
It runs preparation unconditionally, passes its receipt directly to deployment,
and waits for the terminal result. Those operations own source synchronization,
necessary builds, packaging, credentials, release selection and audience.

This invocation and its operation-owned tool traffic are the entire deployment
workflow. No agent-side discovery, file inspection, checks, Git operations,
manual builds, previews or separate hosting calls are allowed. Do not delegate them.

Success is the operation returning `url`; return that literal production URL
and stop. On a returned failure, report it directly and stop. Investigation or
repair requires a separate user instruction.
