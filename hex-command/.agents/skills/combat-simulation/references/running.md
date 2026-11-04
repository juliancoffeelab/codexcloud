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

`--diplomacy unrestricted,diplomatic` filters the modes declared by each campaign's
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

- `scriptEvents`: additional general events, including their objective metadata.
- `scriptConditions`: existing script event IDs mapped to replacement `when`
  expressions. Use these to express route prerequisites and ordering.
- Legacy `goals`, `optionalGoals`, `events` and `eventConditions` overrides are
  rejected; author route objectives through general events.
- `ignoreOptionalWhen`: a condition after which the simulation player omits
  incidental optional-event targets. This changes its goals, not combat legality.

Definitions are explicit: the runner never infers exits, enemy factions, goal
positions or a campaign's intended branches. Selecting a simulation route validates its
name, stages, scripted objectives, events and conditions with the normal campaign validators.
Playable catalog loading does not validate simulation route references.
Simulation setup prepares each selected campaign/route once and shares its immutable
definition across trials and streamed phase requests. Routes that only filter AI
targets borrow the authored definition. Changed definitions copy only the selected
campaign; the rest of the catalog remains shared. Each game reads one supplied
campaign definition throughout restore, retry, combat and mission progression.
Routes add no playable menus, choices, commands or save fields. The browser retains
its authored goals.
