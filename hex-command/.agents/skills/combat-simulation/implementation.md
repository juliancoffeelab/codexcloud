# Combat simulation implementation

Read this when changing or debugging the native harness, Python runner or
analyzer. Normal operation and report-first interpretation are in
[usage.md](usage.md).

The Rust harness owns scheduling, battle execution, campaign transitions and
recording. `scripts/run-combat.py` supplies the current project's data directory,
runs the requested panel, preserves native logs, then invokes
`scripts/report-combat.py` on the saved results. The full default panel covers
all arena policies against Classic on every map; campaign competence uses Mixed
and Priority against the fixed Classic enemy. Campaign players are deliberately not the full arena
policy list.

## Report evidence and limits

The maintained analyzer consumes compact outcomes and, when available, the
referenced full gameplay journal. It supports current report files, historical
raw campaign/arena directories, JSONL journals, and Arrow journals (loading
PyArrow only when needed). It does not rerun battles, inspect source code or
resolve commits/catalog fingerprints.

The findings begin with physical match coverage, compact arena wins by race and
map against Classic, then campaign completion tables. The arena overview keeps
player policies separate, pools recorded opponent races, seats and initiatives,
and excludes identical race/policy self-play. Cells show wins/all games and caps;
missing comparisons are explicit. `race_map_vs_classic.csv` exports those counts.
This fixed-enemy overview serves the balance purpose described in the
[skill purpose and policy roles](SKILL.md#purpose-and-policy-roles).
Stage diagnostics and detailed fixed-enemy/equal-policy matrices follow the
compact tables; the complete report retains every recorded matchup.

For campaigns it writes human-readable completion summaries by campaign and
stage, separating victories, defeats, caps and unclassified endings under their
recorded player, enemy, diplomacy and limit settings. Failure rows use recorded
terminal categories and reasons. With phase journals it also summarizes goal
progress, named goal-actor arrivals, named losses, recruiting, casualties,
damage dealt/taken, empty-player-phase tails, repeated physical tail states,
and defeats with no recorded goal progress. Missing phase records are stated;
the report must not invent inactivity or combat findings for outcome-only data.

These are diagnostics from recorded behavior, not a tactical counterfactual or
proof of root cause. They can identify a narrow repeat pattern and representative
match, but do not determine whether another move was legal, safer or better.
When a routine question needs an aggregate that the report lacks, extend the
maintained analyzer and its tests; do not create one-off scratch parsers or
replace report-first review with manual journal inspection. After the report
identifies a specific unexplained case, inspect that match's relevant phases
and then source as needed. Keep conclusions tied to evidence and distinguish
recorded terminal causes from inferred mechanisms.

Arena grouping uses recorded faction, opponent, map and both policies. Each
physical match contributes one result per participant, including both seats in
self-play. Recruitment, promotion and casualty events belong to their recorded
side. Caps remain in denominators. Duplicate configurations and inconsistent
winner/cap records fail analysis. Baseline comparisons require identical
configurations. Timing and journal-file changes do not imply gameplay changes.

The runner's default full reports use JSONL gameplay details. Direct native
simulation can also record Arrow. The analyzer reads either format; PyArrow is
needed only for Arrow diagnostics. It writes `findings.md`, `summary.md`,
table CSVs and `metadata.json`. The report emits current findings to stdout and
records analyzer elapsed time in `timings.json`. Native aggregate rows own seed-block
confidence intervals; the analyzer does not calculate new intervals.

## Scheduling and execution

The Python runner adds `--vs-classic` by default: every selected arena policy
faces Classic across all selected factions, seats and initiatives. Classic is
added as an opponent even when absent from the selected player policies.
`--all-pairs` keeps the full matrix and `--self-play` keeps same-policy matchups.
Direct native calls retain the all-pairs default unless `--vs-classic` is passed.

Arena scheduling pairs `(race, policy)` entrants across seats and initiatives;
identical entrants avoid duplicate seat permutations. Schedule order is
canonical and independent of worker timing. Campaign scheduling combines
authored campaigns, selected simulation-player policies, the selected enemy
policy and diplomacy settings. The default role-appropriate campaign players
are Mixed and Priority versus Classic enemies.

Arena maps use rotationally paired geometry, equal total budgets and paired
recruitment limits. The leader and starter prices are deducted from the starting
budget. Arena holding counters advance only after completed opposing turns;
losing any required objective resets the counter. Campaign execution delegates
objectives, waves and transitions to the ordinary battle rules.

Per-match records contain phase commands, combat damage/retaliation, casualties,
recruitment, promotions and endpoint states. Phase snapshots retain unit
identity, position, HP, XP, action flags and leader mode, plus gold, ownership,
objective progress and campaign state. These fields support recorded campaign
and phase diagnostics, but do not encode unchosen legal actions or counterfactual
risk. Casualties include troops recruited and killed within the same phase.
Composition summaries are allocated separately; detailed summaries borrow full
trace arrays from the typed result.

## Journals and checkpoints

The final native report is schema 3. Direct simulation defaults to Arrow detail
journals; `run-combat.py` requests JSONL for the default readable diagnostic
workflow. Compact outcomes appear in canonical schedule order and retain byte
ranges and SHA-256 digests into their detail journal. Each Arrow range is an
independently readable IPC stream tagged
`arrow-ipc-snapshot-dictionaries-v2`. Row zero holds the match header and
snapshot dictionaries; subsequent rows hold chapter indices, phase headers,
ordered before/after troop indices and ownership indices. Troop dictionary
entries are typed structs; nullable optional fields restore omission, while
null ownership codes restore explicit JSON null. Ownership columns use the
corresponding chapter's initial ordered tile identities. Header JSON preserves
all remaining data without schema projection. SHA-256 covers exact IPC bytes.
Completed ranges are readable before the batch finishes. `--trace` also
serializes raw typed results to `.trace.jsonl`.

Completed outcomes append once to `.outcomes.jsonl` as
`{"index": <schedule position>, "outcome": <compact outcome>}` in worker
completion order. `.partial.json` is an atomic progress snapshot marked
`"progressFormat": "indexed-outcomes-jsonl-v1"`, containing status, completed
and expected counts, elapsed time, fingerprint, journal filenames and complete
byte lengths. Progress refreshes every ten completions and on completion or
error; it does not copy accumulated outcomes.

`checkpoint::recover_outcomes` reads complete index records, rejecting
duplicates and out-of-range indices. `records::read_detail` validates byte
ranges and checksums before decoding full details. Comparison summaries omit
state hashes while preserving actual states, array order and missing-versus-null
distinctions.

## Full gameplay comparison

`scripts/compare-simulation-reports.py` streams one pair of checksummed journal
records at a time and compares every gameplay field, retaining endpoint hashes,
ordered arrays, unknown fields, missing/null distinctions and numeric
representations. It ignores JSON object key order and only the explicit
commit/fingerprint/timing, journal format and journal-location metadata. It
decodes native Arrow dictionaries losslessly with PyArrow, loaded only for Arrow
inputs. Use it to validate exact policy optimizations.
`scripts/export-simulation-details.py` exports either format as complete JSONL
records without rerunning matches. The report analyzer consumes saved outcomes
and journals to create findings; it does not make counterfactual tactical
claims.

`scripts/convert-simulation-dump.py` reads detail journals with an explicit
Arrow schema. `scripts/compare-simulation-dumps.py` uses a one-to-one Polars
join on `(kind, config)` and preserves array order. Missing or duplicate
identities fail. It excludes `stateHash`, `elapsedMs`, `elapsedSeconds` and
`detail` at match and mission level, and records wall time, CPU time and peak
memory.

Planning internals are in [planner dispatch](../../../rust/src/ai/planner/mod.rs),
[queued search](../../../rust/src/ai/planner/queued.rs) and
[bounded evaluation](../../../rust/src/ai/planner/candidates.rs);
content loading is in [authored game data](../../../rust/data/README.md).
