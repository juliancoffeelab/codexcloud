# Factions and controllers

Every scenario and campaign stage defines its armies in `roster.json` under
`factions`. Each entry owns its ID, controller, palette, relations, starting `gold`,
`recruits` and `units`. Starting units inherit their faction and phase from that
entry; do not repeat `side` or `faction` on roster placements. `map.json` contains
terrain and presentation, not armies. Campaign and map metadata derive from the
same roster; there is no separate `initial` roster.

A faction declares its stable ID, `phase` (`player` or `enemy`), `controller`,
visual `palette`, and diplomatic `relations`. Controllers are `human`, `passive`,
or `ai` with an optional independently selected policy. An omitted AI policy inherits
the session policy. Palette IDs name artwork palettes;
adding a visual palette does not require a new Rust enum. Every AI controller receives its
own planning workspace and can act only with its own units. Controllers sharing
a phase finish before that phase ends; passive controllers issue no actions.
Controller workspaces are transient and are never saved.

Relations are `allied`, `neutral`, or `hostile`. Members of the same faction are
allied. The implicit player and enemy factions are hostile; other relations
default to neutral. A relation can be declared on either faction, but opposing
entries must agree. Attack legality and AI threat/target selection use these
relations, rather than assuming different turn sides are hostile. Neutral units
do not count toward defeating the player's enemies, including `noUnits`
conditions and pending scripted spawns. Healing and defensive auras also respect
faction alliances.

For example, a civilian faction can wait for a neighboring player unit:

```json
{
  "id": "villagers",
  "phase": "enemy",
  "controller": { "kind": "passive" },
  "palette": "passive",
  "transfer": {
    "to": "player",
    "when": {
      "condition": { "kind": "proximity", "side": "player", "radius": 1 },
      "remember": false
    }
  }
}
```

An initial unit belongs to the containing faction entry; a scripted spawn
sets `faction` on its spawn group. Here `enemy` selects the automatic phase, not
hostility. The passive palette is white. On conditional transfer, the existing unit
changes faction and phase, retaining its ID, health, items and physical type. It
retains its existing action flags: a passive unit that has not acted can move
immediately, while a unit that already acted gains no extra actions. The destination faction supplies
control and palette, so transfer to the implicit player makes the unit blue and
controllable without spawning a replacement.

Factions own treasuries and settlements independently of their turn phase.
The roster gives each faction a nonnegative `gold` balance. Runtime snapshots
store those balances in a map keyed by faction ID. A tile's `owner` is
its faction ID. At phase start each faction receives income from its own villages
once. Settlement healing requires the unit's faction to own the tile. Moving onto
unowned or hostile territory captures it; allied and neutral territory is retained.

A recruiting leader must occupy its own faction's keep. An explicit `recruits`
array supplies `{ "unit": "type-id" }` entries, optionally overriding the price
with `"cost": 6`. Omitted prices follow current unit rules. An empty array disables
recruitment until an event unlocks troops. Recruitment spends only that faction's
money and produces a unit in that faction. Generated arenas retain their configured
roster pools. Recruitment quotes do not
change combat unit stats. Native inspection publishes pools, prices and income
by faction; the frontend uses these facts alongside faction palettes and control
permissions for display and input.

Transfer conditions use the shared `ConditionRule` vocabulary. An omitted tile
refers to the unit's current tile. Ownership, event, turn, proximity, and combined
conditions are supported; the transfer itself is the durable result, so there is
no separate transfer-memory record. Escort simulation planning can approach an
unowned nonhostile courier using its authored transfer condition before escorting
it under player control.

An enemy group can declare a local hold order:

```json
{
  "id": "post-guard",
  "order": { "kind": "hold", "area": ["7,6"], "attackRadius": 1 }
}
```

Only units assigned to that group receive the order. They remain inside the hold
area and attack only legal targets within its engagement radius; a displaced unit
approaches its assigned area. The order creates no army-wide survival, escort or
defense duty. Other army units retain their own objectives. Group activation
conditions continue to apply independently.

## Safe evacuation and recruitment

A campaign event may declare `evacuate: { tile, units }`. The unit filter uses
`side` and optional `role` and `name`, independently of promotion/type. The event
waits for a matching unit to occupy the destination, then removes it without a
combat casualty. Its full unit record and the event ID remain in the chapter's
`evacuated` results. Escort/escape completion, required companions and leader
survival recognize these safe outcomes after reload. Matching named companions
and leaders can carry their progression and equipment into the next chapter;
ordinary rescued civilians remain off the battlefield.

Events can also unlock faction-keyed `recruitment` lists. Recruitment availability is derived
from completed event markers, so it persists without a parallel saved unlock
flag and can depend on meeting or retaining a particular character.

## Field-army availability

AI field-army counts exclude passive or unarmed units and units committed to
active local hold orders. A released guard rejoins the available force. A named
group with no active local order remains an ordinary part of its army. This is a
mechanical availability fact shared by planners; their recruitment thresholds and
movement scoring remain independently owned. Leader diagnostics use actual
relations and the same availability facts rather than counting a whole phase.

## Roster authoring

```json
{
  "factions": [
    {
      "id": "player",
      "phase": "player",
      "controller": { "kind": "human" },
      "palette": "player",
      "gold": 14,
      "recruits": [{ "unit": "swordsman" }],
      "units": [{ "type": "swordsman", "col": 0, "row": 5, "role": "leader" }]
    }
  ]
}
```

A full battle also declares its other factions. Campaign carryover adds gold to
the player treasury after loading these balances. Event grants and companion-loss
patches use `"recruitment": { "faction-id": [{ "unit": "type-id" }] }`; grants
unlock entries, while patches replace the listed factions' base pools. Completed
event markers remain the durable unlock facts.
