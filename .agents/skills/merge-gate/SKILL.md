---
name: merge-gate
description: Use when the user asks whether a PR is ready to merge, or asks to merge it.
---

# Merge gate

Give a quick readiness verdict from existing evidence. Budget under one minute
for the audit, including tool calls and quick checks; batch reads and bound
commands to the remaining time. If evidence cannot be checked within that budget,
report the gap. Preparation is separate work, undertaken only when requested.
Routine implementation, pushes and deployment do not invoke this skill.

## Check existing evidence

Identify the PR head and inspect existing review/check results, the changed-file
list and commit history. Reuse evidence for unchanged content; check whether later
changes invalidate it. Assess these four conditions:

- **Independent review:** at least one reviewer other than the implementer covered
  the final substantive change, with findings resolved or given an evidenced
  disposition. An agent review uses fresh context without implementation history;
  a separate human review also counts.
- **Markdown hygiene:** added Markdown and changed guidance were reviewed using
  [writing-for-agents](../writing-for-agents/SKILL.md). Task requirements, plans,
  handoffs and progress reports have not leaked into tracked files. Retained
  documents have an intentional permanent purpose; guidance is current and avoids
  duplicated rules or copies of readily discoverable code/config.
- **History:** each commit has a coherent purpose and accurate message. Unrelated
  changes, accidental artifacts and obscuring WIP/fixup/revert chains were addressed.
- **Validation:** the affected projects' required checks passed for the current
  source, with commands, results and tested revision available. Reliable local
  results count without CI; absent, failed or stale evidence remains a gap.
  Documentation-only changes need relevant documentation checks.

Use quick probes such as diff whitespace checks or small targeted format/link
checks when useful and within budget. A quick probe cannot substitute for a
required suite. Missing evidence becomes a blocker, not an instruction to launch
reviews, install dependencies, run long suites, fix files or rewrite history.

**Complete when:** each condition is supported, failed or unverified for the
identified head, or the time budget is reached.

## Return the verdict

Report **ready** only when all four conditions are supported. Otherwise report
**not ready**, name the concrete blockers or unverified conditions, and give the
next actions. Include the head, evidence used and quick checks run. Keep this
concise and in the response or PR metadata, rather than adding a gate report file.

For an explicitly requested merge, follow repository permission and conflict
rules, confirm the head is unchanged and merge only the ready revision, using an
expected-head guard where supported. A moved head needs reassessment; a blocked
gate returns its verdict without starting preparation. Confirm a successful merge
with the hosting service.

**Complete when:** the user has the verdict and any blockers; an authorized merge
is complete only after hosting-service confirmation.
