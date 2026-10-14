# AI block redesign

Current policy names now select independent Recruiter, Campaign and Planner
blocks. Their preset composition and configurable fields are defined in
[`presets.rs`](../rust/src/ai/presets.rs).
Rust workspaces, browser options and native saved-phase requests can override the
three blocks. Composition remains fixed during a run; browser saves contain
resolved game state rather than planner continuations. The three Old policies
remain frozen historical executables.

The Recruiter owns unit selection and placement. The Campaign block owns mission
duties, essential actors, reservations, corridor clearing, diplomacy, rebuilding
and rebuilding directives. The Planner stages legal commands through Classic,
Full or Queued search. Shared search helpers belong to the selected planner. Tactical algorithms no longer select recruitment
or campaign behavior by the full policy enum.

## Objective fixes

- Escort, Escape, Vigil, typed Seal and actor-specific Capture use one exhaustive
  actor-routing classification. Adding another objective kind requires updating
  that match; it cannot silently fall through an enum wildcard.
- Friendly blockers clear the required actor's corridor. Normal weighted routes
  remain preferred when they permit arrival this phase. Otherwise a short yielding
  route may clear an approach reachable this phase, accounting for the destination
  tile's entry cost. Visited path parents handle zero-cost terrain without loops.
- Classic retains personal actor duties rather than switching to incidental nearby
  objectives. Vigil still requires the authored uninterrupted enemy-turn hold.
- A missing typed-seal specialist is requested by the Campaign block, independently
  of combat recruitment preferences. The leader rebuilds even when its army would
  ordinarily be considered large enough.
- Escalated Refuge objectives discard the shelter destination and target the enemy
  patrol. Planners share terminal-attack classification; Refuge requires the last
  patrol enemy, whereas Defeat requires the remaining commander.

There are no `matches!` invocations in the current AI Rust module, including tests.
The build rejects them, and the module denies Clippy's enum wildcard match lint.
Enum filters use nested exhaustive functions. Frozen historical source is unchanged.

## Campaign verification

Ran **3,200 complete campaign attempts** against fixed Classic: 100 matched
seeds (1001–1100), both diplomacy modes, four campaigns and four player presets.
Each campaign starts at chapter one, earns carryover and receives no retries.
There is one recruit per phase and a 30-round cap per chapter. Caps stay in the
denominator. Battler is arena-only; Old policies are frozen and excluded.

Full campaign completions, **before → after**, out of 200 attempts per cell:

| Campaign      |   Classic |      Rush |     Mixed |  Priority |
| ------------- | --------: | --------: | --------: | --------: |
| Lantern Road  |     0 → 0 |  14 → 186 | 140 → 172 | 200 → 196 |
| Green Accord  |     0 → 0 |     2 → 0 |    8 → 16 |   58 → 76 |
| Ashen Oath    |     0 → 0 | 119 → 123 | 199 → 200 | 199 → 200 |
| Borrowed Tide | 200 → 200 | 198 → 198 | 198 → 200 | 200 → 200 |

Rush on Green Accord (2 → 0) and Priority on Lantern Road (200 → 196) regress
in this sample. The fixes change paths and combat exposure; they do not uniformly
improve tactical strength. Classic still fails three complete campaigns.

Chapter victories / attempts reaching that chapter (conditional, not full-campaign
completion rates):

| Chapter map      | Classic |    Rush |   Mixed | Priority |
| ---------------- | ------: | ------: | ------: | -------: |
| lantern-ferry    |   0/200 | 188/200 | 172/200 |  200/200 |
| green-border     | 200/200 |   6/200 |  80/200 |  172/200 |
| green-vigil      |   0/200 |     0/6 |   16/80 |   78/172 |
| ashen-quarry     | 200/200 | 200/200 | 200/200 |  200/200 |
| ashen-refuge     |   0/200 | 197/200 | 200/200 |  200/200 |
| tide-nets        | 200/200 | 198/200 | 200/200 |  200/200 |
| tide-wreck       | 200/200 | 198/198 | 200/200 |  200/200 |
| tide-sluice      | 200/200 | 198/198 | 200/200 |  200/200 |
| lantern-villages |       — | 188/188 | 172/172 |  200/200 |
| lantern-bell     |       — | 186/188 | 172/172 |  196/200 |
| ashen-dawn       |       — | 123/197 | 200/200 |  200/200 |
| green-pact       |       — |       — |   16/16 |    76/78 |

All capped attempts were screened for five empty player phases or five identical
physical states among their last ten player phases. Remaining flagged cases:

| Player | Chapter      | Attempts |
| ------ | ------------ | -------: |
| rush   | ashen-dawn   |        2 |
| rush   | green-border |       18 |

The reproduced Vigil, escort, missing-specialist and stale-Refuge stalls no longer
appear in this batch. Remaining flagged caps involve defended enemies and poor
individual combat trades: Rush against Green border defenders and the Ashen
commander. A saved Ashen position needs three attacks together (3 + 3 + 4 damage)
to kill a 9-HP commander; each initial trade incurs 7 retaliation. That needs
coordinated combat planning. This sample does not prove that every possible
campaign deadlock is eliminated.

Complete strata, paired gains/losses, chapter denominators, flagged seeds and
native CPU measurements are in [ai-block-redesign.json](benchmarks/ai-block-redesign.json).
Full gzip gameplay journals, endpoints and per-policy metrics remain under
`tmp/ai-block-redesign/campaigns-accepted/`. The production binary and catalog
fingerprints are recorded in the JSON. The baseline uses the same current catalog
and matched configurations, before the block redesign.

To reproduce the fixed-enemy scope, build with `npm run ai:rust:build`, then use
the existing `scripts/compare-cheap-policies.py` module:

```python
import importlib.util
from pathlib import Path

spec = importlib.util.spec_from_file_location("paired", "scripts/compare-cheap-policies.py")
paired = importlib.util.module_from_spec(spec)
spec.loader.exec_module(paired)
configs = [c for c in paired.configurations("campaign", 1001, 100)
           if c["enemy"] == "classic"]
binary = Path("rust/target/release/hex-command-sim").resolve()
Path("tmp/reproduce-ai-blocks").mkdir(parents=True, exist_ok=False)
for policy in ["classic", "rush", "mixed", "priority"]:
    paired.run(binary, policy, configs, Path("tmp/reproduce-ai-blocks") / policy)
```

## Checks and limits

Small regressions cover custom composition through command-at-a-time execution,
missing-specialist recruitment, blocked Vigil arrival and its real hold duration,
the reproduced merfolk escort corridor, and escalated Refuge targets and terminal
attacks. The escort fixture includes promoted units, an injured healer, village
ownership and the already-consumed event, which materially affect the stalemate.

WASM bindings and the shared test runner pass; the shared suites take 14.02 seconds
against the 20-second target. Content validation, UI checks, TypeScript checks,
production client build and formatting pass. Clippy completes with existing style
warnings and no denied enum wildcard matches. The native production build also
checks frozen reference provenance.

This is an intentional campaign behavior change, so historical exact-gameplay
comparisons in [simulation-performance.md](simulation-performance.md) describe the
previous build. No authored content or frozen policy implementation changed.
Campaign completion differences are not controlled speed measurements: changed
outcomes alter the number and difficulty of phases.

Recruitment roles still need the separately documented alignment with infantry-only,
fair use of every roster option and situational selection. These presets do not
establish those balance invariants. Combat planning can still stall when coordinated
attacks would win but each individual trade scores poorly; fixing objective routing
alone does not make those combinations visible to a greedy planner.
