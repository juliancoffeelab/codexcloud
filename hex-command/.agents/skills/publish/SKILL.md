---
name: publish
description: Use when the user requests preparation and saving of an unpublished game release.
---

# Release preparation

Use this save-only workflow when the user requests an unpublished release.
For an intended live human playtest or an explicit deployment request, use the
[deploy skill](../deploy/SKILL.md).

Invoke `node scripts/site-operation.mjs prepare` through the exact
[operation transport](operations.md#code-mode-invocation). Preparation owns
Sites source synchronization from the current committed snapshot (`HEAD`),
necessary builds, packaging, credentials and saving the
exact release. Keep its returned receipt for publication.

Reuse a saved release only for that exact implementation commit and Site; cache
state is an optimization, never a publication prerequisite. No validation suites,
clean-tree checks or archive audits belong to preparation. Dependencies come
from the committed lockfile, not the mutable implementation workspace.

An active automatic-publication window is incompatible with a requested
unpublished release. Preparation must not push into that window; deployment may
reconcile its exact automatic publication instead.

Leave uncommitted implementation changes alone; neither a clean-tree check nor
automatic implementation commits belong to release preparation.

Completion is a returned receipt identifying the saved version and source commit.
Report that it is unpublished. An explicit deploy request goes directly to the
deploy operation, without a separate preparation invocation.
