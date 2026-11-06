---
name: agent-failure
description: Diagnose the cause of an agent failure when the user asks why its behavior or output went wrong.
---

# Agent failure

Find the mechanism that produced the failure and the smallest change that prevents
recurrence. Treat failed instructions, including gaps and ambiguity, as instruction
failures rather than explanations about the agent's diligence or intentions.

## 1. Establish the failure

Read the relevant user decisions, applicable instructions, actions and resulting
artifacts. State the expected behavior and the concrete deviation. Distinguish
instructions that existed from material demonstrably available during the decision.

**Complete when:** the deviation is supported by evidence and its scope is clear.

## 2. Trace the cause

Follow the path from the governing requirement through the decision or workflow
to the observed result. Find where the requirement stopped constraining behavior:

- Missing, ambiguous, contradictory or stale instructions.
- A context pointer or disclosure step that failed to reach necessary guidance.
- A workflow that promoted an assumption into a requirement, or verified the wrong
  contract.
- Tooling, interfaces or data models that permit the invalid operation.

An explicit existing rule must not be relabelled as missing or ambiguous to fit an
explanation. Quoting that rule alone does not establish the cause either. Report
uncertainty and the next discriminating check when the mechanism is unproven.
Claims of forgetting, overlooking, or failing to apply instructions are not causes.

**Complete when:** the causal explanation connects evidence to the deviation, with
observations separated from hypotheses. An unexplained link remains open work.

## 3. Prevent recurrence

Prefer changing code or architecture so the invalid operation cannot be expressed
through the supported interface. If the cause is clearly an instruction defect,
identify the exact instruction to edit or remove; extend it only when necessary.
Avoid accumulating incident-specific prohibitions or duplicating existing rules.

Explain how the proposed change interrupts the demonstrated failure mechanism.
Identify a concrete check that distinguishes prevention from hiding or renaming the
symptom. Apply repairs within the user's authorized scope and respect requested
pauses; a diagnosis does not resume unrelated work.

**Complete when:** the smallest justified correction and its recurrence check are
concrete. Report the failure, evidenced cause, proposed or completed correction,
and any unresolved uncertainty.
