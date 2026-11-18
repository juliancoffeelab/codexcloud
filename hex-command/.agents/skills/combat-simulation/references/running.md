# Running simulations

Read when selecting a schedule, limits, custom tactics or another data catalog.
Work from the game root. Use `node scripts/toolchain/with-toolchain.mjs cargo run --locked --manifest-path rust/Cargo.toml --profile simulation --bin hex-command-sim --features simulation -- --help` for current flags and
defaults. Determine actual coverage from the resulting report metadata.

- Keep experimental catalogs in ignored tmp/ or artifacts/ and pass --data-dir.
  An experiment does not promote content or authorize deployment.
- Choose arena or campaign scope and configurations relevant to the change.
  Defaults are 10 arena seeds and 20 campaign seeds; `--seeds` overrides both.
  Aim for about one minute including incremental compilation and analysis.
  Inspect `--schedule-only`: seeds multiply across maps, armies, tactics, seats
  and campaign routes, so the full matrix has no one-minute guarantee. For
  routine validation, narrow those dimensions before increasing the sample.
  Use full scope for requested broad analysis or evidence that needs it.
  Preserve the selected campaigns' authored diplomacy modes and normal caps.
  Preserve supplied Rust flags and build settings; changing them can rebuild
  dependencies before any simulation runs.
- Faction balance uses matching tactics under equal budgets and normal earned
  progression. A typical scoped run is:

```sh
node scripts/toolchain/with-toolchain.mjs cargo run --locked --manifest-path rust/Cargo.toml --profile simulation --bin hex-command-sim --features simulation -- --mode arena --policies balanced --armies human,woodland --landscapes open --self-play --seeds 10 --unit-usage
```

- --with R,C,P adds a custom recruiter, campaigner and planner combination; a dash
  retains that slot’s workload default. Campaign roles can select it as custom.
  Use --schedule-only to inspect the chosen configurations before expensive runs.
- Keep run files and fingerprints together. Preserve partial output on failure;
  worker errors are not battle defeats. State completed and expected coverage.
- A request to run a report means fresh simulations. For an explicitly requested
  analysis of an existing run, use:

```sh
python3 .agents/skills/combat-simulation/scripts/report-combat.py artifacts/saved/report.json --out artifacts/saved/analytics
```

Use the [metrics reference](metrics.md) to choose required telemetry and the
[reporting reference](reporting.md) to deliver the requested comparison.

## Campaign objective routes

`--diplomacy unrestricted,diplomatic` filters the modes declared in the simulation manifest's
`campaigns.<id>.diplomacy`. `--objectives flight,fight` likewise filters that campaign's
named simulation routes. The scheduler takes their cross product with seeds,
player tactics and enemy tactics. Reports keep both dimensions separate.
`--objectives authored` runs the unchanged authored objectives as a control.

Declare routes under `campaigns.<campaign-id>.routes.<route-name>.stages` in
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

## Fixed-position planner timing

`scripts/benchmark-policy-phases.py --report <report.json> --out <directory>`
uses the report as a configuration source. It runs fresh native matches once and
captures selected planner-entry positions with `--capture-stdio`; it does not
reconstruct or claim to reproduce the report's historical checkpoints. Additional
campaign configurations can come from `--campaigns <details.jsonl.gz>`. Every
provided diplomacy mode and objective route is retained. One seed is selected per
distinct configuration. `--rounds` and `--side` select capture boundaries; output
coverage identifies requested and reached positions.

For comparisons across builds, generate fixtures once, then pass
`--fixtures <first-output>/positions.json` to each benchmark run. Do not regenerate
positions separately with each planner revision. The artifact contains complete
native state (including combat RNG), AI seed, effective objectives, configured
recruitment budget, diplomacy, arena configuration or campaign route, and source
fingerprints. A null recruitment budget means per-controller policy defaults;
zero means no recruiting. Benchmark planner/recruiter overrides are intentional.
The comparison keeps the former one-recruit budget by default; `--recruits N`
changes it, while `--recruits source` retains the captured configured budgets.

Snapshots are captured after turn preparation and before planning. Replay must
use neither `beginTurn` nor `advanceTurn`, which would repeat income, healing and
event effects. Capture is optional, separate from ordinary journals and outside
measured decision intervals. Timing output records the frozen fixture digest and
both source and tested-engine fingerprints. Changes in those fingerprints are
not evidence of an identical historical replay or a performance improvement.

The harness defaults to Balanced against Balanced in arenas and Balanced against
Classic in campaigns, with 10 arena seeds and 20 campaign seeds. Map, faction,
campaign-route and diplomacy coverage remains unchanged. Select broader tactic
coverage explicitly with `--policies classic,rush,balanced,rotated,situational,predictor`
and, when needed, `--campaign-players balanced,rotated,situational,predictor`.
