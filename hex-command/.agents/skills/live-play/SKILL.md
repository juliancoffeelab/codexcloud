---
name: live-play
description: Play the actual game in an isolated browser with the native AI and a disposable Playwright probe.
disable-model-invocation: true
---

# Live play

Use only when the user explicitly authorizes playing the real game in a browser.
A request for ordinary implementation, review, simulation or deployment does not
supply that authorization. A natural-language request to perform live gameplay
can authorize this workflow; invoking the name alone still needs a gameplay scope.

1. Establish the authorized campaign/scenario, assertions and limits from the
   request. Record these in the disposable probe. Read the project testing skill;
   read save-migrations for loading investigations. Keep browser gameplay outside
   the default test suite.
2. Start a local Vite dev server using the project's build/toolchain guidance.
   Use a separate port from concurrent work. Copy
   [the Playwright skeleton](scripts/campaign.mjs) into ignored `tmp/`, and adapt
   that copy to the requested case. Use an isolated context and generated progress;
   use synthetic historical fixtures only when relevant and label their provenance.
3. Set `LIVE_PLAY_AUTHORIZED=1` after establishing authorization. Run the probe
   from the game root. Override `CHROMIUM_PATH` if using an installed system
   Chromium; otherwise use Playwright's installed browser. Configure limits and
   hooks as described below before running. Execution is done when the requested
   assertions have observed their target transitions, or a bounded failure has
   captured enough evidence to identify the blocker.
4. Inspect artifacts and distinguish campaign completion, AI defeat, application
   failure, harness failure and exhausted limits. For loading, compare full native
   checkpoints (including seed), identities, historical entries and browser UI
   before and after restoration/reload; a visible turn number alone is inadequate.
   Report findings and validation limits. Fix application code only if the user
   authorized fixes. Keep current-run evidence under ignored `tmp/`.

## Running the copied skeleton

```sh
# From hex-command; use a unique artifacts directory for every run.
LIVE_PLAY_AUTHORIZED=1 CHROMIUM_PATH=/usr/bin/chromium \
  LIVE_PLAY_URL=http://127.0.0.1:5191 \
  LIVE_PLAY_ARTIFACTS=tmp/live-play/run-1 \
  ENEMY_PAUSE_SCALE=0 PLAYER_AI_POLICY=classic CAMPAIGNS=mirror-night \
  node tmp/live-play/campaign.mjs
```

The skeleton exports `openHarness` and `runCampaigns` for a custom probe. Pass
`playerAI` (or `PLAYER_AI_POLICY` for its policy), `operationTimeout`, `totalTimeout` and `enemyPauseScale` to
`openHarness`. Pass `campaigns` (including repeated IDs for multiple runs),
`maxActions`, `maxRetries`, `enemyAI`, `recall(state)` and `onStep(harness, event)`
to `runCampaigns`. Defaults are bounded and make no victory guarantee. AI defeat
is an observed game result; do not manufacture a win or silently retry indefinitely.
The empty default recall is legal but may reduce competence; adapt it when the
requested workload needs survivors.

`step()` plans with the real WASM `chooseAiCommand` against the live engine's
current revision. It applies moves, attacks, recruits, promotions and end-turns
through the existing controller; AI intent bookkeeping uses native dispatch and
the controller's usual commit. It never substitutes a simulated battle or changes
units, gold, objectives, winners or campaign progression. Dialogue dismissal and
mission transitions use normal controller commands. After a completed campaign it
opens the actual next-game dialog and verifies visibility. Subsequent runs start
through the same controller as the UI. Adapt UI clicks when setup interaction is
itself under test; controller-driven setup does not validate those controls.

The adapter instruments the served Vite controller response to capture a closure
over the current game and inject the existing controller pause dependency. It
requires local Vite source URLs and fails when the constructor marker changes;
it does not support production bundles. Inspect `src/controller/battle.ts`,
`src/game/live-game.ts` and generated engine types before adapting a changed seam.
No tracked application source is rewritten. `ENEMY_PAUSE_SCALE` scales only enemy
presentation pauses (zero still yields); native planning and action computations
run unchanged. Operation and total deadlines remain real wall-clock bounds.

The harness exposes `snapshot`, `entries`, `command`, `evaluate`, `checkpoint`,
`step`, `page` and `log`. Always close it in `finally`. An operation timeout does
not cancel an in-flight browser evaluation: capture available diagnostics and
close the browser instead of continuing that session. Keep extensions read-only
except for normal game commands and explicitly scoped storage fixtures/faults.
Use UI interactions for UI assertions and controller/engine inspection for state
assertions. Record modified source adapters and their limits in the probe.

Artifacts contain action decisions and revisions, terminal states, screenshots,
full native checkpoints, progress metadata, console/page errors and isolated
browser storage. Give each run a fresh directory so appended logs stay unambiguous.
The total deadline closes the browser even if synchronous AI computation hangs;
renderer failure may prevent a final screenshot. Preserve earlier checkpoints and
logs in that case. A clean browser console and a returned snapshot alone do not
prove the requested behavior.
