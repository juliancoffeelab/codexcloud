# Running simulations

Read when selecting a schedule, limits, custom tactics or another data catalog.
Work from the game root. Use npm run ai:simulate -- --help for current flags and
defaults. Determine actual coverage from the resulting report metadata.

- Keep experimental catalogs in ignored tmp/ or artifacts/ and pass --data-dir.
  An experiment does not promote content or authorize deployment.
- Choose arena, campaign or full scope from the request. If scope is unspecified,
  run the full default panel. Preserve the authored campaign diplomacy modes.
- Faction balance uses matching tactics under equal budgets and normal earned
  progression. A typical scoped run is:

```sh
npm run ai:simulate -- --mode arena --policies balanced,situational,predictor --self-play --seeds 50 --unit-usage
```

- --with R,C,P adds a custom recruiter, campaigner and planner combination; a dash
  retains that slot’s workload default. Campaign roles can select it as custom.
  Use --schedule-only to inspect the chosen configurations before expensive runs.
- Keep run files and fingerprints together. Preserve partial output on failure;
  worker errors are not battle defeats. State completed and expected coverage.
- A request to run a report means fresh simulations. For an explicitly requested
  analysis of an existing run, use:

```sh
npm run ai:analyze -- artifacts/saved/report.json --out artifacts/saved/analytics
```

Use the [metrics reference](metrics.md) to choose required telemetry and the
[reporting reference](reporting.md) to deliver the requested comparison.

## Campaign objective routes

`--diplomacy pragmatic,restrained` filters the modes declared by each campaign's
`playerDiplomacy`. `--objectives flight,fight` likewise filters that campaign's
named simulation routes. The scheduler takes their cross product with seeds,
player tactics and enemy tactics. Reports keep both dimensions separate.
`--objectives authored` runs the unchanged authored objectives as a control.

Declare routes under `objectives.<campaign-id>.<route-name>.stages` in
`rust/data/simulation/campaign.json`. Stage keys are authored map identifiers;
route names are arbitrary identifiers, not Rust enum variants. Campaigns without
routes retain one `authored` configuration. Adding a campaign or route needs data
and its normal campaign-list registration, not simulator code.

Each stage can declare:

- `goals` and `optionalGoals`: complete replacement arrays using the existing
  campaign Goal schema. Omitted arrays keep the authored objectives.
- `events`: additional events using the existing campaign Event schema.
- `eventConditions`: existing condition-event IDs mapped to explicit replacement
  conditions. Use these to express route prerequisites and ordering.
- `ignoreOptionalWhen`: a condition after which the simulation player omits
  incidental optional-event targets. This changes its goals, not combat legality.

Definitions are explicit: the runner never infers exits, enemy factions, goal
positions or a campaign's intended branches. Selecting a simulation route validates its
name, stages, goals, events and conditions with the normal campaign validators.
Playable catalog loading does not validate simulation route references.
Routes apply to a private campaign override sharing the native simulation catalog; they add no playable
menus, choices, commands or save fields. The browser retains its authored goals.
