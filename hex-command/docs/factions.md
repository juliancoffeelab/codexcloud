# Factions and controllers

Units can declare a `faction` independently of their turn `side`. A map's optional
`factions` array defines those factions. Existing content without these fields
retains the implicit `player` and `enemy` factions and its selected AI policy.

A faction declares its stable ID, `phase` (`player` or `enemy`), `controller`,
visual `palette`, and diplomatic `relations`. Controllers are `human`, `passive`,
or `ai` with an independently selected policy. Palette IDs name artwork palettes;
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

An initial unit uses `"side": "enemy", "faction": "villagers"`; a scripted spawn
sets `faction` on its spawn group. Here `enemy` selects the automatic phase, not
hostility. The passive palette is white. On conditional transfer, the existing unit
changes faction and phase, retaining its ID, health, items and physical type. It
retains its existing action flags: a passive unit that has not acted can move
immediately, while a unit that already acted gains no extra actions. The destination faction supplies
control and palette, so transfer to the implicit player makes the unit blue and
controllable without spawning a replacement.

The two existing phases still own their treasuries, recruitment pools and tile
ownership. Separate unit factions and controllers do not create separate
territorial economies. Runtime state and inspection facts are published by Rust;
the frontend uses resolved faction palettes, relations and control permissions
for display and input.

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

Events can also unlock a `recruits` list. Recruitment availability is derived
from completed event markers, so it persists without a parallel saved unlock
flag and can depend on meeting or retaining a particular character.
