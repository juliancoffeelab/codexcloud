---
name: agent-failure
description: Diagnose the cause of an agent failure when the user asks why its behavior or output went wrong.
---

# Agent failure

Produce an evidence-based diagnosis and ranked proposed remedies. Do not edit
code, instructions or artifacts as part of this workflow.

## 1. Establish the failure

Establish the expected behavior from the user's request and decisions. Inspect
the actions and resulting artifacts to identify the concrete deviation.

**Complete when:** the deviation is supported by evidence and its scope is clear.

## 2. Trace the cause

Investigate in this order:

1. **Architecture:** what the model, boundaries, interfaces, ownership and workflow
   allow, and what they encourage. Identify whether the unwanted behavior is a
   supported operation or a natural extension of the design.
2. **Existing code:** trace the actual path, its placement, representation, naming
   and implementation patterns. Establish how those choices lead toward the
   failure and what the existing checks verify. For non-code tasks, examine the
   existing artifacts and how they are produced.
3. **Skills and other instructions:** after tracing architecture and code, inspect
   relevant guidance for gaps, ambiguity, contradictions, stale information or
   ineffective context pointers. Distinguish instructions that existed from
   material demonstrably available during the decision.

Follow the path from the governing requirement through these mechanisms to the
observed result. Support relevant layers with evidence; explain when a layer does
not contribute. Do not substitute an instruction explanation for an architectural
or implementation cause.

An explicit existing rule must not be relabelled as missing or ambiguous to fit an
explanation. Quoting that rule alone does not establish the cause either. Report
uncertainty and the next discriminating check when the mechanism is unproven.

**Complete when:** the causal explanation connects evidence to the deviation, with
observations separated from hypotheses. An unexplained link remains open work.

## 3. Propose remedies

Prioritize remedies in the same order as the diagnosis:

1. **Architecture:** remove the invalid operation or change the boundaries and
   defaults that encourage it. Prefer prevention by construction.
2. **Existing code:** simplify or correct the path, placement and implementation
   patterns responsible for the failure.
3. **Skills and other instructions:** where guidance contributes, identify the
   exact instruction to edit or remove; extend it only when necessary.

Do not recommend instruction changes as a substitute for repairing defective
architecture or code. Avoid accumulating incident-specific prohibitions or
duplicating existing rules.

Explain how the proposed change interrupts the demonstrated failure mechanism.
Identify a concrete check that distinguishes prevention from hiding or renaming the
symptom.

**Complete when:** the smallest justified correction and its recurrence check are
concrete. Report the failure, evidenced cause, ranked proposed remedies, and any
unresolved uncertainty. Stop at that report.
