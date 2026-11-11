# Recorded metrics

Read before interpreting composition, damage, support, protection or casualty value.
This is the shared definition of the exported evidence; missing measurements are
unknown rather than zero. Match faction, unit/tier, policy, opponent and map scopes.
For opponent-specific presence claims, use a restricted two-army run or an equivalent
scoped census; pooled presence cannot support an exact-opponent conclusion.

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
node scripts/toolchain/with-toolchain.mjs cargo run --locked --manifest-path rust/Cargo.toml --profile simulation --bin hex-command-sim -- --unit-usage
python3 .agents/skills/combat-simulation/scripts/report-combat.py artifacts/saved/report.json --out artifacts/saved/analytics --unit-usage
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
  Units recorded for a faction elsewhere in the analyzed run or baseline have
  zero presence and undefined rates in scopes where they were not fielded.
  Units never recorded for that faction are omitted. Membership comes from
  recorded faction rows, never from a unit's mechanical race.
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

## Recorded movement paths

Move actions may include `path`: the actual traversed hex IDs, including origin
and destination, resolved by the engine for fog/proximity-sensitive movement.
An omitted path means it was not recorded; it does not mean the unit stayed put.
Use recorded paths for movement overlays rather than drawing straight lines
between move endpoints through terrain the unit did not traverse.
