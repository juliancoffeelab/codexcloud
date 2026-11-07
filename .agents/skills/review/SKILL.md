---
name: review
description: Use when your task is to review a proposed or existing change.
---

# Review

Assess whether the change solves the intended problem and earns its place.
Review requests produce findings; editing the implementation is separate work.
For a merge-readiness decision, use [merge-gate](../merge-gate/SKILL.md).

## Establish scope

Read the user's requirements, applicable instructions and the complete change.
Identify the reviewed revision or working-tree state. Follow affected callers,
consumers and contracts far enough to understand the consequences.
Form your own conclusions before consulting earlier review conclusions. Distinguish
self-review from an independent review by someone who did not implement the change.

**Complete when:** the intended outcome and review scope are explicit, and each
changed area has been inspected in its relevant context.

## Apply four questions

- **Intent:** Does this solve the user's actual problem? Identify missed
  requirements, behavior that contradicts the request and unnecessary scope.
- **Correctness:** What concrete scenario breaks it? Trace boundary conditions,
  failures and interactions with existing behavior. Separate introduced defects
  from pre-existing problems.
- **Simplicity:** Is this the simplest approach that solves the problem? Look for
  existing mechanisms that suffice, unnecessary abstractions and special cases.
  Propose a simpler alternative only when it preserves the required behavior.
- **Repository burden:** Does each added file, dependency, rule and test give more
  value than its total cost? Check for duplication, stale guidance and temporary
  requirements, plans or handoffs leaking into maintained content. For skills and
  agent instructions, apply the [instruction checklist](references/agent-instructions.md).

Resolve meaningful uncertainty by the cheapest useful method: source inspection,
an existing check, manual use or a disposable probe. Follow the affected project's
testing policy. Recommend a permanent test only if you can promise its value exceeds
its total burden, including runtime, maintenance, brittleness and duplication.
Missing permanent tests alone is not a finding. Ground findings in a concrete
consequence; label unresolved suspicions as questions rather than established bugs.

**Complete when:** all four questions have been applied across the change, and
suspected issues have either a supported consequence, a reason for dismissal or
an explicit unresolved question.

## Report

Lead with actionable findings, ordered by impact. For each, give the location,
trigger or scenario, consequence and suggested direction. Include simplification
and burden findings when their benefit is concrete; skip personal style preferences.
Keep questions and verification limits distinct from findings. State when no
findings remain, and identify what was reviewed and any checks actually run.
Keep the review in the response or requested PR review, rather than adding a report
file to the repository.

**Complete when:** the reader can act on each finding and understand the review's
scope and limits.
