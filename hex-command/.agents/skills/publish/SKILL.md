---
name: publish
description: Use to prepare and save an unpublished game release after a checked, pushed implementation batch or an explicit save-only request.
---

# Release preparation

Invoke `node scripts/site-operation.mjs prepare` through the exact
[operation transport](operations.md#code-mode-invocation). Preparation owns
source synchronization, necessary builds, packaging, credentials and saving the
exact release. Keep its returned receipt for publication.

Completion is a returned receipt identifying the saved version and source commit.
Report that it is unpublished. Drafts and simulation-only tasks do not trigger
preparation. For an explicit request to update the live game, use
[deploy](../deploy/SKILL.md) instead.
