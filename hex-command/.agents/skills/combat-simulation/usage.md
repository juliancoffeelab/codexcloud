# Combat simulation usage

Run commands from the `hex-command/` project root. The agent workflow is in
[SKILL.md](SKILL.md); report contracts, counting rules and recording details are in
[implementation.md](implementation.md).

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

`ai:full` runs arenas and campaigns with ten seeds. Use `ai:schedule` to
inspect counts for a selected scope.

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

## Analyze saved reports

```sh
npm run ai:analyze -- artifacts/current/report.json --out artifacts/current/analytics
npm run ai:analyze -- artifacts/current/report.json --baseline artifacts/previous/report.json --out artifacts/current/analytics
```

Open `index.html` for searchable tables, `summary.md` for complete matrices, or
CSV exports for every row. For race balance, use `race_equal_policy.csv` and
`race_map_policy.csv` to compare other factions with matching policies.
`matchups.csv` retains every opponent and policy pairing, including self-play;
`race_all_policies.csv` mixes policies.

## Replay and benchmarks

Replay reruns the recorded matches and requires the original engine/content
fingerprint. It validates journal references before execution:

```sh
npm run ai:simulate -- --replay artifacts/current/report.json
```

`npm run ai:benchmark -- --out artifacts/baseline.json` runs 60 representative
matches. Use `--limit 12` for a smaller sample, or `--compare artifacts/baseline.json`
to check identical complete-result hashes after an optimization.

## Compare full saved gameplay

Convert each detail dump once, then compare the resulting Arrow files. These
commands compare recorded gameplay without rerunning matches:

```sh
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/baseline.arrow artifacts/baseline.details.jsonl
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/current.arrow artifacts/current.details.jsonl
uv run --with pyarrow --with polars==2.0.0 scripts/compare-simulation-dumps.py artifacts/baseline.arrow artifacts/current.arrow
```

See [recording and recovery](../../../rust/src/simulation/README.md) for journal
formats and interrupted runs. AI behavior is described in the
[policy reference](../../../rust/src/ai/README.md).
