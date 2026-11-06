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

## Failures

Diagnose agent failures through instructions, workflow, tooling or architecture.
“I failed to apply/follow the instruction” is not a diagnosis. When the user asks
what caused an agent failure, use [agent-failure](.agents/skills/agent-failure/SKILL.md).

# Skills

- [writing-for-agents](.agents/skills/writing-for-agents/SKILL.md): Use when creating or editing skills or modifying agent instructions.
- [design-audit](.agents/skills/design-audit/SKILL.md): User-invoked requirements, design, and plan-audit workflow.
