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
