---
name: combat-simulation
description: Use for Hex Command AI policy reviews or changes, combat simulations, and balance or campaign competence analysis.
---

# Combat simulation

Work from `hex-command/`. Read [purpose.md](purpose.md) before AI policy reviews
or interpreting balance and campaign results. Read [implementation.md](implementation.md)
only when changing or debugging the harness, analyzer, recording or performance.
Use the [Rust toolchain skill](../rust-toolchain/SKILL.md) for build problems.

## Run

```sh
npm run ai:simulate
```

Cargo builds when needed, runs the full panel and generates the report. Defaults:
10 seeds, all arena tactics against Classic on all maps, and Balanced/Rotated/Situational
campaigns against Classic. Default round caps are 30 for arenas and 100 per
campaign stage; `--rounds N` overrides both. Honor requested scope; otherwise
use this full panel.
Classic uses its live cap of 2 recruits per phase; the other tactics have no
additional recruitment cap. `--recruits N` explicitly overrides both sides.
Each run gets a fresh ignored `artifacts/simulation-*` directory.

After changing the harness, analyzer or AI behavior, run fresh simulations and
present the report in the final reply without another request. Use the affected
workload, or the full panel if scope is unclear. Reporting only that a run or
tests passed does not complete the task.

| Option                                                   | Use                                                          |
| -------------------------------------------------------- | ------------------------------------------------------------ |
| `--mode arena\|campaign\|full`                           | Select the workload.                                         |
| `--seeds N`, `--seed N`                                  | Select seed count and starting seed.                         |
| `--with R,C,P`                                           | Add a custom tactic; `-` uses workload defaults.             |
| `--policies LIST`                                        | Select arena tactics; also filters default campaign players. |
| `--armies LIST`, `--landscapes LIST`, `--campaigns LIST` | Select content.                                              |
| `--campaign-players LIST`, `--campaign-enemies LIST`     | Override campaign roles.                                     |
| `--rounds N`, `--recruits N`, `--diplomacy LIST`         | Override limits or filter authored diplomacy modes.          |
| `--out PATH`                                             | Choose an output directory or raw `.json` file.              |
| `--no-report`                                            | Keep raw results without analysis.                           |
| `--all-pairs`                                            | Request the full arena policy matrix.                        |
| `--self-play`                                            | Compare matching tactics; requires `--mode arena`.           |
| `--schedule-only`                                        | Inspect configurations without running matches.              |
| `--baseline PATH`                                        | Compare with a saved run with identical configurations.      |
| `--help`                                                 | List remaining options.                                      |

`--with=rotation,-,queued` adds an uncapped custom tactic. `rotated` aliases
`rotation`. Slots are recruiter, campaigner and planner; defaults are Balanced
recruitment, Arena for arenas or Mission for campaigns, and Queued planning.
By default it plays against Classic. `--all-pairs` includes it alongside every
selected named tactic; `--self-play` compares each tactic against itself.
Campaign role lists can refer to the added tactic as `custom`, for example
`--campaign-players balanced --campaign-enemies custom`. Without explicit
campaign player selection, the custom tactic is the player.

Use `--data-dir` when simulating another checkout. Determine actual coverage,
limits and composition from report metadata; artwork lineups do not establish it.
Keep run files and fingerprints together. On failure, preserve partial output
and state completed/expected counts; worker errors are not battle defeats.

A request to run a report means fresh simulations, including after analyzer
changes. Analyze an existing run only when that is requested:

```sh
npm run ai:analyze -- artifacts/saved/report.json --out artifacts/saved/analytics
```

## Read evidence

Use the printed findings first. Read `<run>/analytics/findings.md` and
`summary.md` internally when output is truncated or more detail is needed.
Use their recorded recruitment, promotions, casualties, goal progress and
terminal reasons; do not replace the report with scratch analysis of raw files.

If evidence is missing, extend the maintained analyzer and its focused tests,
then regenerate from the saved run. Inspect a targeted phase journal and code
only after the report identifies an unexplained case. Separate recorded failure
from proposed cause. Claim regressions only with matching baseline conditions.

## Present in chat

1. State completed coverage, seeds, round/recruit limits and pooled dimensions.
2. Give **a separate table for each player tactic against Classic**, with races
   as rows and maps as columns. Cells are `78.8% (caps: 13)`: percentages and
   caps, without win counts. Do not combine tactics. Omit Classic-player baseline
   and reverse Classic-versus-player tables unless requested.
   Display Coastal as Merfolk and put Merfolk last. Order map columns as Plains (open), Forests
   (wooded), Crossing (crossings), Islands (islands).
3. Follow with **the campaign table**, campaigns as rows and simulation-player
   tactics as columns. Cells are completion counts such as `16/20`, without
   percentages. Split campaigns with multiple player diplomacy modes into
   separate labeled rows, such as Ashen Oath (pragmatic) and Ashen Oath
   (restrained); never pool these modes. Name the fixed Classic opponent.
4. Add important findings and limitations. For scoped requests, show that scope.

Keep caps in arena win-rate denominators. Campaign completion covers the whole
campaign; stage results count only attempts reaching that stage. Keep wins,
defeats and caps distinct. Summarize both workloads for full runs.
Report files and exports are internal records in ignored `artifacts/`; present
results directly, without local file links. Deliver files only when requested.
