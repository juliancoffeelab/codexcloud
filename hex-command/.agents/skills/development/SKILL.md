---
name: development
description: Use before Hex Command repository work, including implementation, reviews, documentation and Git operations. Explicit deployment requests use deploy directly.
---

# Hex Command development

Use [development references](../../../docs/development.md) for setup and technical
commands. The README is user-owned; modify it only when explicitly asked.

- **Cornerstone rule: never resolve merge or rebase conflicts autonomously.**
  Resolve conflicts only in response to an explicit user request for conflict resolution.
  Do not initiate a merge, rebase, cherry-pick or branch synchronization to address
  known conflicts without that explicit request. Requests to implement, integrate,
  push, publish or prepare a PR do not imply permission to resolve conflicts.
  If conflicts block requested work, report them and stop the affected operation.

- Commit and push completed requested repository changes by default, regardless
  of task duration, through the current feature branch and pull request. Honor
  explicit requests to keep changes local or hold a push. For unfinished work,
  commit and push after more than 10 minutes unless the user requests otherwise.
  Report check status and remaining work.

Deployment eligibility, execution and recovery belong to the [deploy skill](../deploy/SKILL.md).
Use it for explicit deployment requests or an intended live human playtest.
Use [publish](../publish/SKILL.md) for requested unpublished releases.

Diagnose the underlying cause and fix the general rule or mechanism instead of
special-casing the current scenario. Keep changes quick to make and review.

Keep temporary data, experiments and diagnostics in ignored `tmp/`, outside
tracked content. Tracked Markdown documents must contain permanent information;
keep current-work tracking and diagnostics out of tracked Markdown files.

Run `npm run format` to apply project formatting automatically.
Startup should be fast. Lazy-load what is possible. Loading failures must
provide a way to diagnose the error and restart cleanly.

Use [testing](../testing/SKILL.md) for bug fixes and validation. Use
[battlefield-contract](../battlefield-contract/SKILL.md) when changing runtime
data ownership, battlefield rendering or artwork. For AI, simulation harness or
analyzer changes, use [combat-simulation](../combat-simulation/SKILL.md), including
its fresh-run, baseline comparison and reporting workflow.

Continue on the current feature branch and pull request unless the user asks to
change branches. New branches start from the freshly fetched upstream default
branch (`main` here), never a stale local copy. Repository PR and merge rules are
owned by [repository-work](../../../../.agents/skills/repository-work/SKILL.md).
Explicit deploy requests follow the deploy skill's operation-only rule, including
for source synchronization.

## Interruptions

When the user interrupts ongoing work, stop that work and keep it paused. Address
only the interruption and any explicitly requested follow-up. Do not resume the
earlier implementation, checks, commits or deployment until the user asks you to
continue.
