---
name: live-play
description: Use only when the user explicitly authorizes playing Hex Command in a browser; exercise the player UI through a game session.
---

# Live play

Play Hex Command through the interface a player uses. Establish whether the
requested game or workflow can actually be played, and describe its usability.
Implementation, review and deployment requests alone do not authorize gameplay.

Every change to the live game goes through normal player input. Observations
and AI advice leave that game unchanged. Applying commands directly to the live
controller or engine, or manually completing presentation barriers, invalidates
a UI playthrough.
State assertions and DOM checks support the session; they do not replace playing it.
The user can request a separate [engine-probe](../engine-probe/SKILL.md) for an
engine end-to-end probe; it does not complete this UI journey.

## 1. Define the session

Identify the requested URL/revision, scenario, browser, input method and endpoint.
For an unqualified playtest, exercise the UI through an ordinary battle to its
end screen. A focused request can end at its specified workflow. Set execution
bounds and record the scope in ignored `tmp/`.

Default to the game's reduced-motion preference and sound off for long journeys.
Use normal motion and sound for checks of movement, impacts, combat synchronization,
enemy-turn completion and victory transitions. The
[browser reference](references/browser-play.md#presentation-settings) shows setup
and switching between settled actions. Record the settings and the actions
exercised at normal speed; a reduced-motion journey does not establish normal
presentation timing or audible output.

**Complete when:** the intended player journey, endpoint and bounds are explicit;
a short interaction probe cannot be mistaken for a complete game.

## 2. Enter through the UI

Use the requested preview or a local game and the available browser tooling.
Start an isolated session unless saved-progress restoration is part of the scope.
Choose the scenario, settings and any authored debug content through the game's
controls. For Playwright, canvas input, diagnostics or special input requirements,
read [browser play](references/browser-play.md). For AI-guided decisions, read
[AI-assisted UI play](references/ai-assisted-play.md). For save-loading investigations,
also read [save-migrations](../save-migrations/SKILL.md).

**Complete when:** the game is ready through the same entry path a player uses,
and its initial screen and session configuration are recorded.

## 3. Play and observe

Repeat this loop through the requested endpoint:

1. Read the visible board and HUD and choose a useful next action, using a Rust
   AI recommendation when helpful.
2. Perform it with the requested input method, using normal hit testing.
3. Observe the visible result, feedback and the next available action. Confirm
   that the chosen unit, target and controls agree before continuing.

Play the connected workflow: choosing a unit, moving or attacking, inspecting
results, choosing the next unit, recruiting, ending turns and navigating dialogs
as the session requires. Notice blocked controls, confusing state, unexpected
camera/layout changes, missing feedback and delays as well as gameplay outcomes.
Record meaningful actions and their visible results as you go. Wait for each
action's visible completion at the selected presentation settings; change modes
only after it settles. Use normal presentation for the timing checks defined above.
Read-only state can confirm legality or an outcome.

**Complete when:** the requested endpoint is visibly reached, or a concrete
player-facing blocker, defeat or execution limit is captured. A battle's result
and whether the interface worked are separate findings.

## 4. If blocked, investigate

Capture the attempted input, screen, expected result and actual response. Inspect
available diagnostics to distinguish a gameplay restriction from a UI failure.
Reproduce through the same controls when useful; retain evidence from the blocked
attempt. Continue through normal controls if the restriction has a legitimate
player-facing resolution. Otherwise report the blocker at that point.

Keep scratch probes and evidence in ignored `tmp/`, following
[testing](../testing/SKILL.md). Source changes and synthetic storage/fault fixtures
need their own authorized scope; a modified run is separate diagnostic evidence.

**Complete when:** the obstruction has an evidenced explanation or a clearly
stated unresolved cause. An internal workaround never completes the UI journey.

## 5. Report what was played

Identify the tested revision/URL, browser and input method, scenario, meaningful
UI workflows exercised and terminal outcome: requested endpoint reached, player
blocked, defeat, execution limit or tooling failure. Link relevant screenshots,
recordings and action notes. Distinguish observed behavior from diagnostic
inferences and name unexercised paths or device limitations.
State motion/sound settings and distinguish reduced-motion UI completion from
normal animation/audio timing verified on the explicitly exercised actions.

Claim a completed game only after its terminal result was reached through the UI;
claim a completed campaign only after its missions and final transition were.
Close the owned browser session and any owned local server.

**Complete when:** the report shows how far a player got, what prevented further
play if anything, and what the evidence actually establishes.
