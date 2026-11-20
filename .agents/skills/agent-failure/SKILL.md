---
name: agent-failure
description: Diagnose the cause of an agent failure when the user asks why its behavior or output went wrong.
---

# Garden investigation

This skill is about building a better environment for an agent. The agent is
doing its best, and the human is doing their best. Sometimes that is not enough.

We investigate what went wrong in the process of communication between the human
and the agent, and what needs to be weeded out so that similar problems do not
happen again.

Investigate the failure and report its causes and remedies. Do not edit code,
instructions or artifacts during this workflow.

## 1. Triage the problem

Inspect what happened and whether the problem still exists. Check any repairs
already made and establish what they fixed, how, and what remains.

**Complete when:** the problem is triaged and its current state is established.

## 2. Find the causes

Examine these layers in order:

1. Architecture.
2. Existing code or artifacts.
3. Instructions.

Investigate how each shaped the agent's actions and contributed to the failure.
For non-code failures, examine the workflow that produced the artifacts.

**Complete when:** each layer has an evidence-supported explanation of how it
contributed, or why it did not; unresolved contributions remain explicit.

## 3. Propose remedies

Address the remaining causes in priority order:

1. Architecture.
2. Existing code or artifacts.
3. Instructions.

For every relevant remaining cause, explain what to change and how that change
fixes the problem. Account for repairs already completed. If no further change
is needed, explain why those repairs are sufficient.

**Complete when:** every remaining cause is linked to an identified change and
its expected effect, or completed repairs are explained as sufficient.

## 4. Verify that the remedies prevent recurrence

Define the class of failures from the causes in step 2. Walk through the original
failure and a plausible variant as an agent starting a new conversation with
zero knowledge of this one. Use only the environment, artifacts and instructions
that would be available after the proposed remedies were implemented. Identify
where the normal workflow would prevent each failure or catch it before it
produced the wrong result. If either failure can still recur, return to step 3
and revise the remedies.

**Complete when:** the evidence shows that implementing the proposed remedies
would eliminate the whole class of failures for an agent with zero knowledge of
this conversation.
