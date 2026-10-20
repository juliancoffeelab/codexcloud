---
name: combat-simulation
description: Use for Hex Command AI policy reviews or changes, combat simulations, and balance or campaign competence analysis.
---

# Combat simulation

Work from `hex-command/`. Read [design.md](design.md) for army design principles
and [purpose.md](purpose.md) before AI policy reviews or interpreting balance
and campaign results.
Use the [Rust toolchain skill](../rust-toolchain/SKILL.md) for build problems.

## Run

Keep experimental content in an ignored copy of `rust/data`, under `tmp/` or
`artifacts/`, and pass that copy with `--data-dir`. Register experimental maps
inside that copied catalog only. Running, rendering or evaluating an experiment
does not authorize adding it to the playable catalog or deploying it. Read the
[publish skill](../publish/SKILL.md) only when the user requests publication.

```sh
npm run ai:simulate
```

For faction balance across the fair-roster and adaptive recruiters, use
matching tactics:

```sh
npm run ai:simulate -- --mode arena --policies balanced,situational,predictor --self-play --seeds 50 --unit-usage
```

Situational scores observed enemies only. Predictor uses the same distance-free
valuation plus a fading prior over legal future recruits, including their
weapon pressure, effective HP and thorn interactions. Both retain movement
value for village capture and terrain reachability, with no proximity discount,
nearby-enemy healing gate, or nearest-origin exchange projection. Select both
with `--policies situational,predictor`; `--with predictor,-,queued` selects the
Predictor recruiter independently of the other AI blocks.

Inspect individual opposing-faction matchups, not only pooled race averages.

Balance investigations must collect `--unit-usage`, or battlefield presence plus
the relevant battlefield-value/support telemetry. This is required evidence,
not an optional presentation request. Start composition analysis with living
unit-turn presence and actual contributions. Pair purchase flow with presence
and casualties: repeated replacement purchases can reflect churn rather than
a large or strong field army. Neither purchases alone nor purchase/presence
ratios establish survival without cohort evidence. Keep matchup scopes aligned;
current presence exports pool opposing factions. For opponent-specific claims,
use restricted two-army runs or an equivalent maintained scoped census; never
pair pooled presence with exact-opponent purchases as though scopes matched. If required
evidence is missing, collect it before making the claim.

Cargo builds when needed, runs the full panel and generates the report. Defaults:
50 arena seeds and 100 campaign seeds, all arena tactics against Balanced on all maps, and Balanced/Rotated/Situational/Predictor
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

| Option                                                   | Use                                                                            |
| -------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `--mode arena\|campaign\|full`                           | Select the workload.                                                           |
| `--seeds N`, `--seed N`                                  | Override both workload seed counts and select the starting seed.               |
| `--with R,C,P`                                           | Add a custom tactic; `-` uses workload defaults.                               |
| `--policies LIST`                                        | Select arena tactics; also filters default campaign players.                   |
| `--armies LIST`, `--landscapes LIST`, `--campaigns LIST` | Select content.                                                                |
| `--campaign-players LIST`, `--campaign-enemies LIST`     | Override campaign roles.                                                       |
| `--rounds N`, `--recruits N`, `--diplomacy LIST`         | Override limits or filter authored diplomacy modes.                            |
| `--out PATH`                                             | Choose an output directory or raw `.json` file.                                |
| `--unit-usage`                                           | Add per-unit damage/presence tables, battlefield presence and support effects. |
| `--no-report`                                            | Keep raw results without analysis.                                             |
| `--vs POLICY`                                            | Fixed arena opponent; Balanced by default. Campaign opponents stay separate.   |
| `--all-pairs`                                            | Request the full arena policy matrix.                                          |
| `--self-play`                                            | Compare matching tactics; requires `--mode arena`.                             |
| `--schedule-only`                                        | Inspect configurations without running matches.                                |
| `--baseline PATH`                                        | Compare with a saved run with identical configurations.                        |
| `--help`                                                 | List remaining options.                                                        |

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

## Purchase shares

The analyzer exports `purchase_shares.csv`, using only recorded new recruits:
family purchases / all purchases for the same faction, tactic, opposing tactic
and map. Starting units, earned promotions and same-faction games are excluded;
opposing factions are pooled. The `all` map rows sum purchase counts across maps.
Families come from report metadata, never the current checkout. Older reports
without family metadata retain individual unit identifiers. Purchase share is
not field composition, survival rate, damage efficiency or a unit win rate.
Purchases measure recruitment flow and can be driven by replacement churn.
Present presence and actual contributions first, then interpret purchases
alongside that exposure and casualties. A purchase-only report cannot support
a composition or unit-strength conclusion.

## Battlefield presence and support effects

```sh
npm run ai:simulate -- --unit-usage
npm run ai:analyze -- artifacts/saved/report.json --out artifacts/saved/analytics --unit-usage
```

Use `--battlefield-presence`, `--support-effects`, `--battlefield-value` or
`--leader-audit` to request those measurements independently. Saved reports must
contain the requested evidence; missing measurements are not zero effects.

Exports:

- `battlefield_presence.csv`: family shares of living non-leader unit-turns,
  measured at each own-phase end, including starting units and promotions.
- `battlefield_units.csv`: the same census separated by exact unit/upgrade.
- `army_usage.csv`: purchases, presence, attack and legacy damage shares together.
- `unit_damage.csv`: exact-unit/tier presence share, weapon damage share, damage
  per living unit-turn and D/P = damage share / presence share. Weapon damage
  includes retaliation, excludes overkill, and excludes leaders from both shares.
  Effective personal/aura thorns are credited to their source and reported
  separately as `thornDamage` and `thornDamagePerUnitTurn`; they never inflate
  weapon D/P. The weighted faction baseline uses total weapon damage divided by
  total nonleader unit-turns. Thus 10% presence and 50% damage means 5.00× D/P.
  Unobserved units have zero presence and undefined rates, rather than zero DPS.
  Damage-free factions have undefined D/P. Catalog units come from recorded
  profiles; promotion tiers remain separate.
  Rows sort by recorded unit role: infantry, flank, ranger, support, then family
  and tier. The `unitKind` column maps warrior to infantry and healer to support.
  Older profiles without a role appear last as unknown; never infer their role
  from current checkout data.
- `unit_damage_changes.csv` and `baseline_unit_damage.csv`: matching per-unit
  damage tables when `--baseline` is supplied. Comparison cells in findings show
  baseline → current; missing rates remain undefined. Each map/tactic/opponent
  scope has its own faction baseline. All-map shares use summed counts.
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

Use actual HP damage when comparing damage output; distinguish weapon damage
from personal and aura thorns. Protection can be negative for vulnerabilities.
Purchase gold and casualty replacement value answer different questions, and
neither establishes an earned promotion's full value.

Capture, control and occupation measure different contributions. Leader exposure
is a forecast, not proof that a death was avoidable or that another position
would satisfy the mission. Read it alongside the remaining army and objectives.

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
alone does not establish that thorns lack defensive value. Saved-phase replays
restore combat RNG from recorded attack and retaliation draws and verify the
final RNG. Strict replays must reproduce the original actions, units and gold
before interpreting an intervention; unsupported draw histories fail explicitly.

Use game data and a simulator matching the saved report. If deliberately testing
a different version, state that limitation. Paired replays measure one-phase
changes in attacks, targets, reflection and casualties; they do not establish
long-term win effects. Spread samples across battles and seeds.

Presence, support and battlefield-value arena exports exclude same-faction
games and pool other opposing factions. `unit_events.csv` also retains
same-faction games; filter those explicitly for faction comparisons. All-map
shares use summed counts. Battlefield presence weights survival through living
unit-turns; battle presence means a unit appeared at any point. Neither is a
unit win rate.

Provider turns include zero-benefit opportunities. Distinguish actual healing
from adjacency, and effective thorn damage from overkill. Damage shares credit
aura reflection to the defender; support and battlefield-value exports credit
the aura provider. Use the same definition when comparing units or reports.

## Read evidence

Use the printed findings first. Read `<run>/analytics/findings.md` and
`summary.md` internally when output is truncated or more detail is needed.
Use their recorded recruitment, promotions, casualties, goal progress and
terminal reasons; do not replace the report with scratch analysis of raw files.

If evidence is missing, collect it before drawing a conclusion. Separate recorded
failure from proposed cause. Claim regressions only with matching baseline
conditions. Preserve completed results if reporting fails; report completed and
expected coverage separately.

## Present in chat

### Required elements

1. State completed coverage, seeds, round/recruit limits and pooled dimensions.
2. Default to **one faction-versus-faction matrix per map**, with row factions
   winning against column factions. Show every recorded opposing faction;
   pooled race averages must not replace the matchup breakdown. Order maps as
   Plains (open), Forests (wooded), Crossing (crossings), Islands (islands).
   Order factions as Humans, Skeletons, Woodland, Merfolk (recorded as Coastal).
   Put comparable policies in each cell in the order **Balanced → Situational →
   Predictor**, using the available subset (for example Situational → Predictor
   when Balanced is absent). State the order and that values are win percentages
   with cap counts in parentheses, for example `55.0 (2) → 49.8 (9) → 70.8 (14)`.
   With one policy, show its single value. Keep caps in the denominator and omit
   win counts.
   Name the row and opponent tactics for each panel. Pair only comparable scopes:
   Balanced-versus-Balanced → Situational-versus-Situational →
   Predictor-versus-Predictor for faction-balance self-play, or all selected row
   tactics against the same fixed opponent for a fixed-opponent panel. Explain when both sides change policy; this does not
   measure head-to-head recruiter strength. Present other tactic panels separately,
   including Classic when recorded. Mark missing configurations as unrecorded;
   do not fabricate a full matrix or broaden a scoped request.
   Mark same-faction diagonals as **Mirror**. Follow with a separate mirror table
   of first-to-act win percentages and cap counts, using physical-game
   denominators, in the same Balanced → Situational → Predictor order using
   the available subset. Do not
   present the mechanically symmetric faction win rate as mirror balance evidence.
3. Follow with **the campaign table**, campaigns as rows and simulation-player
   tactics as columns. Cells are completion counts such as `16/20`, without
   percentages. Split campaigns with multiple player diplomacy modes into
   separate labeled rows, such as Ashen Oath (pragmatic) and Ashen Oath
   (restrained); never pool these modes. Name the fixed Classic opponent.
4. Add important findings and limitations. For scoped requests, show that scope.

Keep caps in arena win-rate denominators. Campaign completion covers the whole
campaign; stage results count only attempts reaching that stage. Keep wins,
defeats and caps distinct. Summarize both workloads for full runs.

### Optional elements on request

These are optional presentation elements; the presence/contribution evidence
required for balance investigations above must still be collected and read.
Include the requested optional elements alongside the required summaries for
the requested scope. Use the meanings below unless the user specifies another
format. A request for a **damage table** means the Presence / Damage-per-turn /
D/P table below, with thorns separate. Weapon-roll tables and detailed damage
accounting exports are distinct outputs.

| Requested element                           | Meaning and evidence                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Damage tables**                           | Per exact unit/tier: Presence %, weapon Damage/turn, D/P and separate Thorns/turn, from `unit_damage.csv`. D/P is weapon damage share divided by presence share: 10% presence and 50% damage is 5.00×. Both shares exclude leaders; weapon damage includes retaliation and excludes overkill and thorns. Thorn damage is effective HP removed, credited to personal/aura sources. Include unused recorded-catalog units as 0% presence and undefined rates (—). Use `unit_damage_changes.csv` for baseline → current comparisons. |
| **Damage shares / combat usage**            | Each family's share of recorded attacks and damage, alongside recruitment and battlefield presence, from `army_usage.csv`. These legacy damage shares include retaliation and overkill and credit aura reflection to the defender; they are not the actual-HP damage table above.                                                                                                                                                                                                                                                 |
| **Recruitment / purchase tables**           | New recruits as counts and shares of purchases, from `purchase_shares.csv`. Starting units and earned promotions are excluded. This measures purchasing choices, not field composition or effectiveness.                                                                                                                                                                                                                                                                                                                          |
| **Battlefield presence**                    | Living nonleader unit-turn counts and shares at own-phase end, including starting units and promotions, from `battlefield_presence.csv` or exact-unit `battlefield_units.csv`. Longer survival contributes more observations; presence is not a unit win rate.                                                                                                                                                                                                                                                                    |
| **Support effects / overlap**               | Actual HP healed, personal/aura reflection, effective reflection, covered allies and overlapping coverage, from `support_effects.csv` and `support_overlap.csv`. Include provider-turn denominators and zero-benefit opportunities; adjacency alone is not healing or damage.                                                                                                                                                                                                                                                     |
| **Objective and casualty value**            | Captures, control and occupation turns, purchased recruit gold lost and casualty replacement value, from the battlefield-value exports. Keep these measures separate: ownership can persist after a captor dies, and replacement value includes free starters and earned promotions.                                                                                                                                                                                                                                              |
| **Additional matchup / policy comparisons** | Breakdowns beyond the default per-map faction matrices, such as additional tactic pairings, seat/initiative splits or baseline changes. Use `matchups.csv` and policy summaries; keep caps in denominators and identify the compared scopes.                                                                                                                                                                                                                                                                                      |
| **Pooled arena summaries**                  | Factions as rows and maps as columns, pooling the other opposing factions and excluding mirrors. Keep tactics separate and name the opponent policy. These optional summaries supplement the default full matchup matrices; they do not replace them.                                                                                                                                                                                                                                                                             |
| **Campaign failure / stage tables**         | Victories among attempts reaching each chapter, defeats, caps, failed goals and recorded terminal reasons, from campaign stage and failure exports. Later-stage success is not whole-campaign completion.                                                                                                                                                                                                                                                                                                                         |
| **Before/after comparisons**                | Changes against a matching baseline in matchups, campaign completion, recruitment or contributions. State matching seeds, schedules and limits, and distinguish observed changes from proposed causes.                                                                                                                                                                                                                                                                                                                            |
| **Leader safety audit**                     | Army strength, leader position/HP, forecast exposure, alternatives and death-boundary evidence, from `leader_audit.csv` and `leader_deaths.csv`. An unsafe forecast does not prove an avoidable death or a mission-compatible escape.                                                                                                                                                                                                                                                                                             |
| **Performance tables**                      | Recorded chooser/phase CPU and workload coverage, using timing exports or a matching benchmark. Separate simulation work from reporting/build time and state the measured workload.                                                                                                                                                                                                                                                                                                                                               |
| **Weapon-roll / theoretical damage tables** | Per-strike ranges, strike counts, expected totals and kill odds under declared modifiers, from `damage-tables.csv` or matching engine forecasts. These describe possible attacks, not units' observed battlefield contributions.                                                                                                                                                                                                                                                                                                  |

For damage or contribution rates, show the exposure denominator, such as living
unit-turns. Keep faction, unit/tier, tactic, opposing tactic and terrain scope
explicit; identify pooled dimensions. Collect the necessary telemetry when it
is absent rather than substituting another table or treating missing data as zero.

Report files and exports are internal records in ignored `artifacts/`; present
results directly, without local file links. Deliver files only when requested.
