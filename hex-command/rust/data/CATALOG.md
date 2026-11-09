# Engine-owned scenarios and workloads

`manifest.json` is the single registry read by Rust for native and browser
engines. Scenario geometry and rosters live in `game/scenario/`; campaign stages
keep their maps, rosters, scripts and story together in `game/campaigns/`.
Frontend code receives resolved metadata through the engine API.

Every `scenario.json` and `campaign.json` declares `human: true` or `false`.
The engine exposes complete metadata for rendering and loading all registered
content, including historical map revisions. The frontend filters new-game
selection lists by `human`; this does not restrict loading or debug launchers.
Standalone scenarios can declare `characters` when their roster contains named
actors. Map identifiers and revisions remain the engine's loading keys.

Workload manifests reference these registered definitions:

- `simulation/arena.json` defines named arena groups, their fixed map variants,
  capture territory, holding duration and default schedule. Each variant either
  selects match armies with an explicit total budget or retains the scenario's
  forces and economies. Positive match seeds cycle through variants in manifest order; seed zero
  selects the first variant. The seed still independently controls combat and AI decisions. Terrain is loaded from
  the scenario, with no runtime generation or fallback template.
- `simulation/campaign.json` defines default campaign selection, diplomacy modes
  and simulation-only objective routes. These settings do not change the
  human-playable campaign definitions.
- `simulation/rl.json` selects existing arena groups, roster pairs, objective
  modes, campaign routes, opponents and recruiters for mixed learning. Python
  obtains this validated definition from Rust and adds sampling/seeding budgets.

The four frozen cases per original arena group preserve the old generator's
seeds 1–4, including their budgets. Identical boards share one map file. Other
seeds select from those cases; they do not reproduce the old procedural layouts.
The event-free Villages and Sluice combat scenarios are registered alongside the
other maps rather than rebuilt by a separate training loader.

Catalog schema changes are independent of saved-battle formats. Immutable old
experiment catalogs remain paired with their recorded engine; new training runs
use the current catalog and a policy warm start when provenance changes.
