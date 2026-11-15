---
name: agent-failure
description: Diagnose the cause of an agent failure when the user asks why its behavior or output went wrong.
---

# Agent failure

Produce an audit of the failure and its current status, a deep causal diagnosis,
and concrete steps to fix the remaining causes. Investigate enough to support
those conclusions; keep verification methods internal rather than making a
verification plan a deliverable or handing investigation back to the user. Do not
edit code, instructions or artifacts as part of this workflow.

## 1. Establish the failure

Establish the expected behavior from the user's request and decisions. Inspect
the actions and resulting artifacts to identify the concrete deviation. Trace
repairs already made and explain what they fixed, how, and what remains. A past
failure does not by itself establish a current defect.

**Complete when:** the deviation, its scope, completed repairs and any remaining
problem are clear.

## 2. Trace the cause

Investigate in this order:

1. **Architecture:** what the model, boundaries, interfaces, ownership and workflow
   allow, and what they encourage. Identify whether the unwanted behavior is a
   supported operation or a natural extension of the design.
2. **Existing code:** trace the actual path, its placement, representation, naming
   and implementation patterns. Establish how those choices lead toward the
   failure and whether subsequent repairs removed that path. For non-code tasks,
   examine the existing artifacts and how they are produced.
3. **Skills and other instructions:** after tracing architecture and code, inspect
   relevant guidance for gaps, ambiguity, contradictions, stale information or
   ineffective context pointers. Distinguish instructions that existed from
   material demonstrably available during the decision.

Follow the path from the governing requirement through these mechanisms to the
observed result. Support relevant layers with evidence; explain when a layer does
not contribute. Do not substitute an instruction explanation for an architectural
or implementation cause.

Explain the failure with an evidence-backed causal chain: what allowed or
encouraged the action, how that mechanism operated in this case, and how it
produced the observed deviation. Admissions such as “it's my fault,” “I ignored
the instruction” or “I jumped to conclusions” must not substitute for that
explanation. Responsibility is not causation; do not invent an internal motive
to fill an evidentiary gap.

An explicit existing rule must not be relabelled as missing or ambiguous to fit an
explanation. Quoting that rule alone does not establish the cause either.
Investigate unresolved causal links; distinguish supported conclusions from
hypotheses and state material uncertainty that remains.

**Complete when:** the causal explanation connects evidence to the deviation,
with observations separated from hypotheses. Continue investigating material
causal gaps while relevant evidence is available; do not finalize with an
admission or a promise to investigate later. If a cause cannot be established,
identify the specific unavailable evidence and explain what remains unknown
without presenting a hypothesis as the cause.

## 3. Propose remedies

Consider remedies at the layers implicated by the diagnosis:

1. **Architecture:** remove the invalid operation or change the boundaries and
   defaults that encourage it. Prefer prevention by construction.
2. **Existing code:** simplify or correct the path, placement and implementation
   patterns responsible for the failure.
3. **Skills and other instructions:** where guidance contributes, identify the
   exact instruction to edit or remove; extend it only when necessary.

Do not recommend instruction changes as a substitute for repairing defective
architecture or code. Avoid accumulating incident-specific prohibitions or
duplicating existing rules.

Propose changes for demonstrated remaining causes, accounting for completed
repairs. Rank fix steps by impact on those causes and make each actionable: name
what to change, where, and how it interrupts the failure mechanism. A possible
future regression alone does not justify more rules or checks. If no further
change is justified, say why the completed repair is sufficient.

**Complete when:** the report explains what failed, what is already fixed and how,
the deep cause, and concrete fix steps for what remains, ordered by impact. State
material unresolved uncertainty and stop at that report.
