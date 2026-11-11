---
name: engine-probe
description: Use when a user requests extensive end-to-end probe using engine, that doesn't require complete UI coverage.
---

# Engine probe

Use on request for an extensive end-to-end probe driven through Rust/controller
interfaces, with only the UI coverage the task requires. “Engine-probe it” is a
sufficient plain-language request; the user need not select this skill. A blocked
live-play session does not authorize changing its boundary, and an internal
workaround cannot complete its UI journey.

## 1. Define the journey

Identify the starting scenario, connected workflows, expected outcome and endpoint
for the requested probe. State which steps use engine/controller operations and
which, if any, need UI interaction. Record the assertions that establish the
end-to-end result.

Default to reduced motion, sound off and no extra enemy pacing. These settings
shorten controller journeys while preserving normal commands, persistence and
completion barriers. Request normal presentation explicitly when its timing is
under test; the [harness reference](references/harness.md) shows the overrides.

**Complete when:** the journey, endpoint and required coverage are explicit.
Complete UI coverage belongs to [live-play](../live-play/SKILL.md).

## 2. Exercise the journey

Follow [verification](../verification/SKILL.md). Use the actual Rust engine and authored
content, with the existing [WASM setup](../../../tests/setup-wasm.mjs) and
[factory helpers](../../../tests/native-fixture.mjs) where useful. Controller
integration can use [its public operations](../../../frontend/game/Session.mli).
For a browser-connected controller probe, read the [harness reference](references/harness.md)
and use its OCaml script variant. Keep disposable probes and artifacts in ignored `tmp/`.

Apply commands through the connected journey and compare state across its
meaningful transitions, including expected rejection where applicable. Use an isolated game and storage
context. Identify synthetic fixtures, fault injection and omitted presentation
explicitly. For persistence claims, follow
[save-migrations](../save-migrations/SKILL.md).

**Complete when:** the requested endpoint and assertions are checked, or a
concrete blocker, execution limit or environment failure is captured. A partial
run remains incomplete.

## 3. Report the boundary and result

Report the tested revision, fixture, endpoint reached, commands, expected and
observed state, and any unresolved question. Label this an **engine/controller
probe** and name the UI steps actually exercised and those omitted. When paired
with live play, keep their evidence and conclusions separate.
Record motion, sound and pacing settings. Reduced-motion results do not verify
normal animation, audible output or audio/impact synchronization.

**Complete when:** the result is reproducible and its scope cannot be mistaken
for a UI playthrough.
