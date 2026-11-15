# Repository instructions

This repository is a playground for experimenting with the Codex cloud feature.

Each project folder is independent. Keep its code, dependencies, configuration
and documentation self-contained; do not introduce cross-project imports or
shared code.

Keep the top-level README short and focused on the repository's purpose. Put
project-specific setup and implementation documentation in the project folder.
For Hex Command work, follow [its AGENTS.md](hex-command/AGENTS.md).

Keep always-applicable instructions in `AGENTS.md`. Put task-specific workflows
and their procedures in skills; list their invocation conditions below.

READMEs are user-owned; modify them only when explicitly asked.

## Repository changes

Use feature branches and pull requests for all repository changes, including
documentation and quick fixes. Do not commit or push directly to `main` or the
repository's default branch. Requests to push or publish do not authorize bypassing
the pull request workflow. Merge a pull request only when the user explicitly
asks to merge it.

Commit and push completed requested changes by default through the current
feature branch and pull request. Honor explicit requests to keep changes local
or hold a push. For unfinished work, commit and push after more than 10 minutes
unless the user requests otherwise; report check status and remaining work.
Continue on the branch and PR you have created unless asked to make a new PR. New
branches start from the freshly fetched upstream default branch (`main`), never
a stale local copy.

## Conflicts

**Cornerstone rule: never resolve merge or rebase conflicts autonomously.**
Resolve conflicts only in response to an explicit user request for conflict
resolution. Do not initiate a merge, rebase, cherry-pick or branch synchronization
to address known conflicts without that explicit request. Requests to implement,
integrate, push, publish or prepare a PR do not imply permission to resolve
conflicts. If conflicts block requested work, report them and stop the affected
operation.

## Working files

Keep temporary data, experiments and diagnostics in ignored `tmp/`, outside
tracked content. Tracked Markdown must contain permanent information; keep
current-work tracking and diagnostics out of tracked Markdown files.

## Interruptions

When the user interrupts ongoing work, stop that work and keep it paused. Address
only the interruption and explicitly requested follow-up. Resume the earlier
implementation, checks, commits or deployment only when the user asks to continue.

## Failures

When the user asks what caused an agent failure, use
[agent-failure](.agents/skills/agent-failure/SKILL.md).

# Skills

- [working-with-subagents](.agents/skills/working-with-subagents/SKILL.md): Use when delegating work to subagents or coordinating their execution.
- [review](.agents/skills/review/SKILL.md): Use when your task is to review a proposed or existing change.
- [merge-gate](.agents/skills/merge-gate/SKILL.md): Use when asked whether a PR is ready to merge or asked to merge it.
- [writing-for-agents](.agents/skills/writing-for-agents/SKILL.md): Use when creating or editing skills or modifying agent instructions.
- [design-audit](.agents/skills/design-audit/SKILL.md): User-invoked requirements, design, and plan-audit workflow.
