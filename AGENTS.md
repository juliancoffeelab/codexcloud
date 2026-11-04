# Repository instructions

This repository is a playground for experimenting with the Codex cloud feature.

Use feature branches and pull requests for all repository changes, including
documentation and quick fixes. Do not commit or push directly to `main` or the
repository's default branch. Requests to push or publish do not authorize bypassing
the pull request workflow. Merge a pull request only when the user explicitly
asks to merge it.

Each project folder is independent. Keep its code, dependencies, configuration
and documentation self-contained; do not introduce cross-project imports or
shared code.

Keep the top-level README short and focused on the repository's purpose. Put
project-specific setup and implementation documentation in the project folder.
For Hex Command work, follow [its AGENTS.md](hex-command/AGENTS.md).

Keep always-applicable instructions in `AGENTS.md`. Put task-specific workflows
and their procedures in skills; list their invocation conditions below.

# Skills

- [writing-for-agents](.agents/skills/writing-for-agents/SKILL.md): Use when creating or editing skills, modifying agent instructions, or diagnosing agent failures.
