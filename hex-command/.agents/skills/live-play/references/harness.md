# Live-play harness

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
and inject its existing pause dependency. Inspect `src/controller/battle.ts`,
`src/game/live-game.ts` and generated engine types if the constructor seam changes.
Keep tracked application source unchanged. `ENEMY_PAUSE_SCALE` changes only enemy
presentation pauses; planning and action computation remain real, and deadlines
remain wall-clock bounds. Record adapter changes and their limits in the probe.

## Saved progress and loading probes

Campaign play is independent of saved-progress support. Artifacts contain
`entries: null` when that API is absent; `entries()` then fails explicitly.
Unavailable history is not an empty successful loading result. Keep extensions
read-only except for normal commands and explicitly scoped storage fixtures/faults.
