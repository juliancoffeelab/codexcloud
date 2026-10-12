# Combat simulation guide

Run simulations as needed to explore or validate balance. Follow the selected
races, maps, policies, seeds and limits. The native harness runs the same battle
rules and AI code as the browser. For the agent workflow, use the
[combat simulation skill](../.agents/skills/combat-simulation/SKILL.md).

## Run a panel

```sh
# Enumerate the full schedule without running matches.
npm run ai:schedule -- --mode full --seeds 10

# Simulate a complete Mixed-policy panel and analyze it.
npm run ai:combat -- --policies mixed --seed 61 --seeds 50 --out artifacts/current

# Restrict races, policies and maps.
npm run ai:combat -- --armies human,coastal --policies classic,mixed --landscapes crossings --seeds 2 --out artifacts/coast-panel

# Native arena or campaign output without the Python analysis step.
npm run ai:simulate -- --mode arena --seeds 2 --out artifacts/arena.json
npm run ai:simulate -- --mode campaign --seeds 5 --out artifacts/campaigns.json
```

`ai:full` runs arenas and campaigns with ten seeds. With the current four factions,
four maps and three policies, that is 11,520 arena matches and 720 campaign attempts.
The schedule command shows counts for a selected scope.

| Option                   | Meaning                                                           |
| ------------------------ | ----------------------------------------------------------------- |
| `--seed`, `--seeds`      | First seed and seed count; defaults 1 and 2                       |
| `--policies`             | `classic`, `rush`, `mixed`                                        |
| `--armies`               | `human`, `woodland`, `undead`, `coastal`                          |
| `--landscapes`           | `open`, `wooded`, `crossings`, `islands`                          |
| `--rounds`, `--recruits` | Round cap and recruit limit per phase; defaults 30 and 1          |
| `--workers`              | Up to eight workers; default follows CPU quota                    |
| `--diplomacy`            | Campaign settings `pragmatic`, `restrained`; defaults to both     |
| `--data-dir`             | Authored data directory; use it when selecting another checkout   |
| `--trace`                | Also save raw typed results; detailed gameplay is always recorded |

Use a fresh output directory per experiment. Keep each report and its adjacent
journals together. Inspect `.partial.json` for status, completed and expected
counts. Preserve partial results after a failure and state how much completed.

## Analyze saved reports

```sh
npm run ai:analyze -- artifacts/current/report.json --out artifacts/current/analytics
npm run ai:analyze -- artifacts/current/report.json --baseline artifacts/previous/report.json --out artifacts/current/analytics
```

Analysis reads saved reports and executes no matches. Baselines require identical
configurations. Open `index.html` for searchable tables, `summary.md` for complete
matrices, or the CSVs for every row. `full-data.zip` contains the exports and
available compact outcome journals; complete gameplay remains in `.details.jsonl`.

For race balance, use `race_equal_policy.csv` and `race_map_policy.csv` to compare
other factions with matching policies. `matchups.csv` retains every opponent and
policy pairing, including self-play. `race_all_policies.csv` mixes policies.
Caps remain in denominators. Mirrored faction cells refer to the same physical
matches; self-play counts both participants. Campaign comparisons include
completion and reached mission results.

## Replay and full gameplay comparison

Replay reruns the recorded matches and requires the original engine/content
fingerprint. It validates journal references before execution:

```sh
npm run ai:simulate -- --replay artifacts/current/report.json
```

To compare full saved gameplay without rerunning it, convert each detail dump
once, then compare the resulting Arrow files:

```sh
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/baseline.arrow artifacts/baseline.details.jsonl
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/current.arrow artifacts/current.details.jsonl
uv run --with pyarrow --with polars==2.0.0 scripts/compare-simulation-dumps.py artifacts/baseline.arrow artifacts/current.arrow
```

`npm run ai:benchmark -- --out artifacts/baseline.json` runs 60 representative
matches. Use `--limit 12` for a smaller sample, or `--compare artifacts/baseline.json`
to check identical complete-result hashes after an optimization.

See [simulation implementation](../.agents/skills/combat-simulation/implementation.md)
for analyzer contracts, arena/campaign definitions and statistics;
[recording and recovery](../rust/src/simulation/README.md) for journal formats;
and [AI policy implementation](../rust/src/ai/README.md) for planning behavior.
Historical balance evidence is in [the initial pass](combat-balance.md) and
[the coastal follow-up](coastal-balance.md). Current stats live in
[authored game data](../rust/data/README.md).
