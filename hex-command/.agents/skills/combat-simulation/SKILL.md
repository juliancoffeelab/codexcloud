---
name: combat-simulation
description: Use for Hex Command AI policy reviews or changes, combat simulations, and balance or campaign competence analysis.
---

# Combat simulation

Work from `hex-command/`. Read [design.md](design.md) for army design principles
and [purpose.md](purpose.md) before AI policy reviews or interpreting balance
and campaign results. Read [implementation.md](implementation.md)
only when changing or debugging the harness, analyzer, recording or performance.
Use the [Rust toolchain skill](../rust-toolchain/SKILL.md) for build problems.

## Run

```sh
npm run ai:simulate
```

For faction balance against the adaptive recruiter, use matching tactics:

```sh
npm run ai:simulate -- --mode arena --policies situational --self-play --seeds 50
```

Inspect individual opposing-faction matchups, not only pooled race averages.

Cargo builds when needed, runs the full panel and generates the report. Defaults:
10 seeds, all arena tactics against Balanced on all maps, and Balanced/Rotated/Situational
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

| Option                                                   | Use                                                                             |
| -------------------------------------------------------- | ------------------------------------------------------------------------------- |
| `--mode arena\|campaign\|full`                           | Select the workload.                                                            |
| `--seeds N`, `--seed N`                                  | Select seed count and starting seed.                                            |
| `--with R,C,P`                                           | Add a custom tactic; `-` uses workload defaults.                                |
| `--policies LIST`                                        | Select arena tactics; also filters default campaign players.                    |
| `--armies LIST`, `--landscapes LIST`, `--campaigns LIST` | Select content.                                                                 |
| `--campaign-players LIST`, `--campaign-enemies LIST`     | Override campaign roles.                                                        |
| `--rounds N`, `--recruits N`, `--diplomacy LIST`         | Override limits or filter authored diplomacy modes.                             |
| `--out PATH`                                             | Choose an output directory or raw `.json` file.                                 |
| `--unit-usage`                                           | Add battlefield presence, combat-use shares and actual support-effect analysis. |
| `--no-report`                                            | Keep raw results without analysis.                                              |
| `--vs POLICY`                                            | Fixed arena opponent; Balanced by default. Campaign opponents stay separate.    |
| `--all-pairs`                                            | Request the full arena policy matrix.                                           |
| `--self-play`                                            | Compare matching tactics; requires `--mode arena`.                              |
| `--schedule-only`                                        | Inspect configurations without running matches.                                 |
| `--baseline PATH`                                        | Compare with a saved run with identical configurations.                         |
| `--help`                                                 | List remaining options.                                                         |

`--with=rotation,-,queued` adds an uncapped custom tactic. `rotated` aliases
`rotation`. Slots are recruiter, campaigner and planner; defaults are Balanced
recruitment, Arena for arenas or Mission for campaigns, and Queued planning.
By default its arena opponent is Balanced; campaigns retain Classic. `--all-pairs` includes it alongside every
selected named tactic; `--self-play` compares each tactic against itself.
Campaign role lists can refer to the added tactic as `custom`, for example
`--campaign-players balanced --campaign-enemies custom`. Without explicit
campaign player selection, the custom tactic is the player.

Use `--data-dir` when simulating another checkout. Determine actual coverage,
limits and composition from report metadata; available sprites do not establish it.
Keep run files and fingerprints together. On failure, preserve partial output
and state completed/expected counts; worker errors are not battle defeats.

A request to run a report means fresh simulations, including after analyzer
changes. Analyze an existing run only when that is requested:

```sh
npm run ai:analyze -- artifacts/saved/report.json --out artifacts/saved/analytics
```

## Purchase shares and fixed-position scores

The analyzer exports `purchase_shares.csv`, using only recorded new recruits:
family purchases / all purchases for the same faction, tactic, opposing tactic
and map. Starting units, earned promotions and same-faction games are excluded;
opposing factions are pooled. The `all` map rows sum purchase counts across maps.
Families come from report metadata, never the current checkout. Older reports
without family metadata retain individual unit identifiers. Purchase share is
not field composition, survival rate, damage efficiency or a unit win rate.

For a targeted scorer investigation, `--phase-stdio` requests can set
`inspectRecruitment: true`. The response includes pre-phase Situational
recruit/promotion score components, eligibility and affordability. Diagnostics
are read-only; normal phase execution remains unchanged.

## Battlefield presence and support effects

```sh
npm run ai:simulate -- --unit-usage
npm run ai:analyze -- artifacts/saved/report.json --out artifacts/saved/analytics --unit-usage
```

The analyzer also accepts `--battlefield-presence`, `--support-effects`,
`--battlefield-value` or `--leader-audit` independently. This audit reads checksummed journals and recorded
`unitProfiles`, never the current catalog. Support effects require fresh runs
with `supportTelemetryVersion`; legacy missing measurements are not zeros.

Exports:

- `battlefield_presence.csv`: family shares of living non-leader unit-turns,
  measured at each own-phase end, including starting units and promotions.
- `battlefield_units.csv`: the same census separated by exact unit/upgrade.
- `army_usage.csv`: purchases, presence, attack and damage shares together.
- `support_effects.csv`: per-unit and all-unit healing, adjacency, overlapping
  coverage, reflected damage and effective reflected damage.
- `support_overlap.csv`: coverage and overlap conditioned on provider count and
  available recipients, so single-provider turns do not hide overlap between
  multiple providers.
- `battlefield_value.csv`: actual weapon HP damage, signed armour/terrain
  prevention, captures, occupation/control turns and casualty gold, by family.
  Includes arena and reached campaign chapters, with mixed campaign armies
  separated by the recorded unit race.
- `battlefield_unit_value.csv`: the same contributions separated by exact unit
  and promotion, so shield and heavy branches are not hidden in family totals.
- `leader_audit.csv`: phase-boundary leader HP/tile, nonleader army strength,
  recruitment availability and reachable-tile exposure forecasts.
- `leader_deaths.csv`: death-boundary evidence linked to that leader's last
  own-phase decision within the same match/chapter.

Fresh native `--unit-usage` runs include the value and leader exports. Older runs
retain their original presence/support exports; requesting either newer audit
explicitly requires its recorded telemetry version. Missing measurements are
never substituted with zeros.

Protection includes racial and unit flat modifiers, attributed before terrain.
Both raw stage deltas and HP-capped prevention are recorded for the initiating
attack and executed weapon retaliation. Values are signed: vulnerability is
negative prevention. Effective weapon damage excludes overkill, reflected thorns
and drain healing. Total HP damage additionally credits effective reflection to
its intrinsic/aura sources, with separate weapon and thorn subtotals. Recruit gold lost counts actual purchases within this arena
or chapter; casualty replacement value uses the dead unit's current catalog cost,
including free starters and earned promotions. Neither is a complete economic
valuation of an earned promotion.

Captures credit the mover that changed ownership. Control turns credit that last
captor while ownership persists, including after its death; occupation turns
credit the living unit standing on the tile at its own phase end. Objective tiles
are the union of capture objectives actually published for that side during the
chapter. Already-owned initial tiles have no inferred captor. These measure
contribution, not proof that a capture or a unit caused victory.

Leader army value is the sum of nonleader unit cost times remaining HP fraction.
Exposure sums independent enemies' legal next-turn attacks, using the existing
planner forecast; it is conservative rather than a coordinated battle replay.
Reachable alternatives hold other units fixed at own-phase start and may not
fulfill mission requirements. Start-board chosen-tile risk and actual end-board
risk are separate fields. The death export includes remaining army, its observed
peak fraction, enemy strength and the last decision's risk. It does not label
every exposed leader death reckless or every late death unavoidable.

For sampled aura deterrence, replay observed enemy-phase positions with aura
enabled and disabled, leaving intrinsic thorns unchanged:

```sh
python3 .agents/skills/combat-simulation/scripts/measure-thorn-deterrence.py \
  --report artifacts/saved/report.json --data-dir rust/data \
  --out tmp/thorn-deterrence --limit 50
```

Use `--effect intrinsic` or `--effect all` to disable personal thorns or both
effects instead of aura alone. `--policy situational` filters the acting tactic;
`--per-seed 10` spreads observations across seeds. `--engaged-only` requires
an affected recipient within an enemy melee unit's geometric movement reach;
this conservative filter does not prove a legal path or an actual attack.
Compare reflection with target/weapon switches and casualties. Low reflection
alone does not establish that thorns lack defensive value.

The explicit catalog and native binary must match the report's fingerprint and
recorded profiles. An explicit `--allow-fingerprint-mismatch` permits weaker
profile-only validation and labels that limitation. Outputs include paired
requests/responses, attack/target/weapon changes, reflection and HP/casualty
differences. These are one-phase counterfactuals, not long-term win attribution;
the first observed positions are correlated within battles. Full profiles,
turn-start gold and ownership are preserved, without repeating begin-turn effects.

Presence, support and battlefield-value arena exports keep faction, tactic,
opposing tactic and map separate, exclude same-faction games, and pool other
opposing factions. Raw `unit_events.csv` also retains same-faction games; filter
those explicitly for faction comparisons. All-map shares use summed counts. Battle presence means a family appeared at any recorded point, including
initial units that died before their first census; field share weights survival
through unit-turns. Neither measure is a unit win rate. Army damage shares retain
native combat attribution: aura reflection belongs to the defender in that
legacy total; the support export attributes it to its actual aura source.

Healing provider-turns and adjacency are sampled at own-turn start; intrinsic
thorns and aura opportunities at opposing-turn start after friendly positioning.
Provider-turns include zero-effect opportunities. Healing records use actual HP
restored after terrain healing and caps. Tied strongest sources split credit;
weaker overlapping providers receive no effect credit. Intrinsic reflection and
aura reflection are separate. Raw reflection includes overkill; effective
reflection is the additional HP removed beyond ordinary weapon retaliation,
with that marginal amount attributed proportionally to intrinsic and aura
strength. Source IDs and damage are captured at combat time, including killing
hits and aura providers that later die. Thorns trigger only while defending
against an initiating melee attack, not against retaliation when attacking.

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
2. Give **a separate table for each player tactic against Balanced**, with races
   as rows and maps as columns. Cells are `78.8% (caps: 13)`: percentages and
   caps, without win counts. Do not combine tactics. Include the Classic-player comparison; omit reverse
   Balanced-versus-player tables unless requested. Use an explicitly requested
   alternative opponent when supplied. Exclude all same-faction games from these
   faction tables so every tactic faces the same three opposing factions. Report
   same-faction tactic comparisons separately when useful.
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
