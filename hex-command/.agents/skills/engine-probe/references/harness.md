# Engine-probe harness

Read when customizing policies, recalls, hooks, loading assertions or the Vite
adapter. Use [campaign.mjs](../scripts/campaign.mjs) for current exports and option
names; the default procedure is in [SKILL.md](../SKILL.md).

## Custom probes

Use `openHarness` and `runCampaigns` for custom probes. Configure bounded limits,
policies, recalls and hooks for the requested workload. The empty default recall
is legal but can reduce competence; select survivors when the workload needs them.
AI defeat is a game result, not permission to retry indefinitely or manufacture a win.

`step()` plans with real WASM against the current revision and applies commands
through the normal controller. Native dispatch owns AI intent bookkeeping.
Dialogue and transitions use normal commands; successful campaign completion is
checked through the actual next-game dialog. Controller-driven setup does not
validate its UI controls: adapt clicks when those controls are under test.

## Adapter changes

The adapter instruments the served Vite controller response to capture the game
and inject its existing pause dependency. The upstream script targets its `src/controller/battle.ts`; the OCaml variant
targets `frontend/game/Session.ml` and its generated module. Inspect the selected
implementation and generated engine types if the constructor seam changes.
Keep tracked application source unchanged. `ENEMY_PAUSE_SCALE` changes only enemy
presentation pauses; planning and action computation remain real, and deadlines
remain wall-clock bounds. Record adapter changes and their limits in the probe.

## Saved progress and loading probes

Campaign play is independent of saved-progress support. Artifacts contain
`entries: null` when that API is absent; `entries()` then fails explicitly.
Unavailable history is not an empty successful loading result. Keep extensions
read-only except for normal commands and explicitly scoped storage fixtures/faults.

## OCaml adapter

Copy `campaign-ocaml.mjs` alongside `campaign.mjs` and import its `openHarness`
and `runCampaigns` exports for the OCaml port. It instruments generated
`Session.js`, retains the live session across battle replacements, and translates
boundary values through compiler fixture codecs. Inspect `frontend/game/Session.ml`
and its emitted module if the constructor marker changes. Compare the same
rendering mode on both applications; use same-application controls for renderer
noise when comparing pixels.

The shared runner accepts `installAdapter({page, config, errors})` to replace the
upstream controller constructor seam, `prepareContext(context)` for isolated storage or
browser fixtures before navigation, and `viewport` for desktop/mobile probes.
These hooks do not change controller commands, AI policies or gameplay rules.

Copy the selected script(s) into ignored `tmp/` before adapting them. The OCaml
variant needs `campaign.mjs` alongside `campaign-ocaml.mjs`. Configure
`ENGINE_PROBE_URL`, `ENGINE_PROBE_ARTIFACTS`, `CHROMIUM_PATH`, `CAMPAIGNS` and the
scope options exposed by the runner. `ENEMY_PAUSE_SCALE` applies only to this
engine/controller probe; record omitted or accelerated presentation in its report.
