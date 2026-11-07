---
name: merge-gate
description: Use before merging a pull request, or when the user asks to assess or prepare merge readiness.
---

# Merge gate

Establish readiness from actual review and validation evidence for the PR's
current source. This workflow does not authorize a merge; follow the repository's
merge-permission and conflict rules. Routine implementation, pushes and deployment
do not invoke this workflow.

## 1. Establish the review scope

Read the request, applicable instructions, complete PR diff and commit history.
Identify the base and head revisions, affected projects and intended outcome.
Reuse current evidence; investigate changes since earlier reviews or check runs.

**Complete when:** the final diff, history and evidence refer to identifiable
revisions, and every affected project has a defined validation scope.

## 2. Obtain an independent review

Require at least one reviewer who did not implement the change. Give an agent
reviewer a fresh context with no implementation history (`fork_turns: "none"`),
the user requirements, applicable instructions and the source/diff to inspect.
Keep prior review conclusions out of the initial brief so it forms its own
findings. A separate human review can also satisfy this requirement.

Reconcile findings against the source and requirements. Fix substantiated defects
or record why a finding does not apply; unresolved defects block readiness.
Reuse review of unchanged content, but obtain independent follow-up for substantive
changes made afterward. Reviewer agreement alone does not establish correctness.

**Complete when:** an independent review covers the final substantive change,
its reviewed revision is recorded, and each finding has an evidenced disposition.

## 3. Review Markdown and history hygiene

Inspect every added Markdown file and changed instruction/document section using
[writing-for-agents](../writing-for-agents/SKILL.md). Task requirements inventories,
plans, acceptance proposals, handoffs, progress logs and diagnostic reports belong
in the conversation, PR metadata or ignored scratch space unless the user explicitly
wants a maintained repository artifact. Judge content, not filenames: retain
intentional durable contracts, necessary provenance and useful operating guidance.
Remove stale instructions, duplicated rules and cheaply rediscovered code/config
copies; verify retained paths, links and ownership. A useful fact does not require
another standalone document or skill.

Review the PR commits in order. Each must earn its place as a coherent change or
necessary integration, with a message matching its actual purpose. Flag unrelated
work, accidental artifacts and WIP/fixup/revert chains that obscure the delivered
change. Report concrete history cleanup; inspecting history does not authorize
rewriting published commits or resolving conflicts.

**Complete when:** every added document has a justified permanent purpose or is
removed, changed guidance is current, and every commit has a clear reason to remain.

## 4. Validate the current source

Run the affected projects' required tests and checks locally: use their instructions
and the changed scope to select formatting, types, content, build and behavioral
validation. Documentation-only work needs the relevant documentation checks, not
unrelated application suites. Honor the project's testing policy when adding tests.

Record commands, results and the tested revision. Reuse reliable passing evidence
for unchanged source; rerun affected checks after fixes. Missing CI is not a pass:
report local results explicitly, and keep readiness blocked when required checks
fail or cannot be run. Passing probes do not replace required suites.

**Complete when:** all required checks pass for the final source, with no undisclosed
failures, skipped requirements or stale evidence.

## 5. Report readiness or merge

Summarize the reviewed/tested head, independent-review disposition, Markdown and
history findings, and local check results. Keep evidence in the response, PR metadata
or ignored scratch space rather than a committed gate/handoff report.

For a readiness request, return the result and blockers. For an authorized merge,
confirm the head has not changed and merge that reviewed revision, using the expected
head SHA where supported. Reconcile new changes before retrying a moved-head merge.
Readiness alone does not authorize deployment.

**Complete when:** readiness is supported by all four preceding stages, or the
specific blockers are explicit; an authorized merge is complete only when confirmed
by the hosting service.
