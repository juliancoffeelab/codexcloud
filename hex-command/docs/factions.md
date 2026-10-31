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
  "transferOnProximity": { "by": "player", "to": "player", "radius": 1 }
}
```

An initial unit uses `"side": "enemy", "faction": "villagers"`; a scripted spawn
sets `faction` on its spawn group. Here `enemy` selects the automatic phase, not
hostility. The passive palette is white. On proximity transfer, the existing unit
changes faction and phase, retaining its ID, health, items and physical type. It
has spent its actions for the current turn. The destination faction supplies
control and palette, so transfer to the implicit player makes the unit blue and
controllable without spawning a replacement.

The two existing phases still own their treasuries, recruitment pools and tile
ownership. Separate unit factions and controllers do not create separate
territorial economies. Runtime state and inspection facts are published by Rust;
the frontend uses resolved faction palettes, relations and control permissions
for display and input.
