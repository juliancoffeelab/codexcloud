---
name: combat-simulation
description: Run Hex Command combat simulations for balance work and analyze saved results with complete comparisons and readable reports.
---

# Combat simulation

Tell the user when applying this skill. Work from the `hex-command/` project root.
Use the existing runner and analyzer. See [usage.md](usage.md) for commands and
CLI options, [implementation.md](implementation.md) for internal contracts and
recording details, and the [Rust toolchain skill](../rust-toolchain/SKILL.md) for builds.

## Scope

Run relevant simulations when exploring or validating balance; do not wait for
separate permission. Follow the user's selected races, maps, policies, seeds and
limits. For unspecified scope, start with the default two-seed arena batch;
include campaigns when relevant. `ai:full` runs arenas and campaigns with ten seeds.
Documentation or analyzer-only work does not need a new balance batch.

Before running, inspect `rust/data/simulation/rosters.json` and `arena.json`.
Establish the leaders, screens, recruitment pools and reachable promotions.
Artwork lineups do not establish which units participate. Campaigns have their
own authored rosters. State the selected scope and coverage before interpreting
balance. Use `ai:schedule` to enumerate configurations when useful.

## Run or analyze

Preserve each batch in a fresh ignored artifact directory. Use existing npm
commands to build and invoke the native harness. Record which data and engine
were used; pass `--data-dir` when using data from another checkout.

```sh
npm run ai:combat -- --seeds 10 --out artifacts/current
npm run ai:analyze -- artifacts/current/report.json \
  --baseline artifacts/previous/report.json --out artifacts/current/analytics
```

The second command analyzes saved results. For long runs, report progress from
`.partial.json`. Preserve partial output on failure and state completed match
counts rather than presenting failed runs as complete.

## Present results

Show all recorded factions and every requested map and policy pairing. Do not
replace complete results with selected opponents or individual duels. Include
unit recruitment, promotions, casualties, per-match results and paired changes.

For race balance, lead with `race_equal_policy.csv`; use `race_map_policy.csv` for
one policy's terrain results and `matchups.csv` for every opponent pairing.
`race_all_policies.csv` mixes policies. Keep caps in denominators, distinguish
self-play from independent matches, and label sample sizes and unresolved results.

Link `analytics/index.html`, `summary.md` and `full-data.zip`. Link original
`.details.jsonl` journals when full raw gameplay is requested. State recorded
metadata, completed match counts, seed ranges and scope limitations. Retain the
raw runs and all exports; do not commit generated artifacts.
