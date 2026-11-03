---
name: repository-work
description: Use before repository work, including reviews, edits, documentation and Git operations.
---

# Repository work

This repository is a playground for experimenting with the Codex cloud feature.

Use feature branches and pull requests for all repository changes, including
documentation and quick fixes. Do not commit or push directly to `main` or the
repository's default branch. Requests to push or publish do not authorize bypassing
the pull request workflow. Merge a pull request only when the user explicitly
asks to merge it.

Each project folder is an independent project. Keep its code, dependencies,
configuration, and documentation self-contained. Projects must not depend on
each other; do not introduce shared code or cross-project imports.

Keep the top-level README short and focused on the repository's purpose. Do not
fill it with project-specific setup instructions, implementation details, or
progress notes. New documentation will usually belong in the relevant project's
own README.md; create one there when needed.

For Hex Command work, use its [development skill](../../../hex-command/.agents/skills/development/SKILL.md).
Explicit deployment requests enter the [deploy skill](../../../hex-command/.agents/skills/deploy/SKILL.md)
immediately; its operation owns preparation, without repository preflight checks.
