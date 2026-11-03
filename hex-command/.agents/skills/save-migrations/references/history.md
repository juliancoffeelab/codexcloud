# Save migration history

Version-specific compatibility notes for the save format. Consult the relevant
section when changing or investigating a historical conversion.

## Version-one compatibility

Both previous formats were version one. TS snapshots have a top-level `version`
and store ownership without terrain. Native envelopes have `engine: "rust-wasm"`
and a version inside `saved`. The v1-to-v2 step distinguishes these formats and
removes their format markers. Historical layouts, defaults and campaign
conversions live exclusively under `src/save/migrations/legacy/`.

TS saves tagged `campaignDeploymentFormat: "named-waves-v1"` already use authored
wave IDs. Untagged published saves from `3956f5c` and `7a30040` translate numeric
markers through each wave's explicit `legacyIndex`. Revision-one Lantern village
and Ashen finale saves already contained subsequently delayed reinforcements;
the importer marks those waves deployed without changing wounded troops, gold
or IDs. Revision-two saves retain pending waves.

The unpublished intermediate draft `1acc1dd` reused a revision with a different
Ashen numeric order. Its format cannot be distinguished reliably from an older
published save, so that ambiguous draft is not supported. Wave identity is never
inferred from surviving troop health.

## Version-two migration

The v2-to-v3 step removes the need for historical layouts in the live catalog.
Revision-one refuge checkpoints retain their own tiles, ownership and
units, while their revision tag is changed to the current catalog entry with the
same geometry. Normal Rust restoration validates the
result. The catalog accepts only current authored documents; old layouts, marker
indices and dialogue are stored under `src/save/migrations/legacy/data/`.

## Version-three migration

The v3-to-v4 step grows the developer map's published 6×20 and 13×23 layouts to
the current 32×32 revision. Existing tiles, ownership, units, battle progress
and seed stay intact; only missing tiles are added from the current map.
It never resets units or respawns defeated showcase troops.
Malformed or incomplete historical boards still fail validation. Other maps
receive only the envelope-version update.

These historical intermediate conversions may carry an opaque `planner` field
through the migration chain. The v4-to-v5 step discards it before native
restoration. Carrying those bytes does not permit interpreting or resuming them.

## Version-four migration

The v4-to-v5 step drops the entire historical `planner` field, including all
queued actions, pending commands and continuation bookkeeping. It preserves
resolved battle state and the pre-turn replay marker. Migration never executes
AI actions or deserializes the historical planner. Normal gameplay creates a
fresh plan when needed.

Older native saves may contain resolved state halfway through an enemy phase.
They have no pre-turn state to roll back to: completed actions remain, while
unexecuted actions are discarded. Their converted data is persisted immediately
after validation, without playing any remaining enemy actions.
This is recovery of a historical format, not permission to save partial phases.
New saves must persist only the boundaries of the entire enemy turn, even when
that turn contains many units and commands.

## Version-five migration

The v5-to-v6 step turns historical implicit type-based leadership into explicit
`role: "leader"` metadata on unit instances. It uses frozen pre-conversion type
IDs stored with the migrations and preserves every existing explicit role.
Current unit definitions have no leadership flag. Rust, AI, UI and current saves
use only the unit's role; ordinary units of the same type do not become leaders.

## Version-six migration

The v6-to-v7 step makes faction data explicit. A frozen conversion fixture supplies
The Silent Crossing's wildlife controller and assigns its existing animals to that
faction, preserving identity, wounds, equipment and progress. Other maps retain
explicit faction data or receive an empty list. Current snapshots always serialize
`factions`, including an empty list, and Rust rejects omitted fields or missing
required authored factions instead of backfilling from the current map.

Older conversion steps read migration-owned metadata frozen from deployed source
`14371387185340e64c5b68706faa409ebe16fdf8`. That fixture is historical conversion
data; do not regenerate it when current maps, unit rules or campaigns change.

## Version-seven migration

The v7-to-v8 step adds the required `groundItems: []` battlefield inventory.
Version-seven checkpoints had carried equipment but no ground items. Conversion
preserves all existing units, equipment and campaign progress. Current snapshots
serialize ground items explicitly, including an empty list; Rust does not invent
missing state during restoration.

## Version-eight migration

The v8-to-v9 step converts Toll Road revision-three checkpoints to revision four
using migration-owned historical encounter identities. It removes uncontacted
preplaced Sella, records an already-completed meeting’s spawn as resolved, and
preserves a joined Sella and every other existing patrol unit. Existing camp
ownership is retained. The old proximity light memory is discarded, swamp entry
memory is translated to the shared encounter marker, and the original mire
caster receives its authored name without changing identity or progression.
Other chapters receive only the envelope update. No runtime restoration backfill
or live catalog lookup performs this conversion.

## Version-nine migration

The v9-to-v10 step writes previously implicit terrain orientations explicitly:
road, signpost, bridge and ford use `e+w`, villages use no connections, and landing/ferry
use `w`. Existing explicit connections are preserved. These are frozen historical
constants inside the migration. Current data, validation and rendering never
supply a missing connection mask.

## Version-ten migration

The v10-to-v11 step moves the published Bell chapter's revision-one/two battle
onto the valley using a frozen, injective coordinate mapping. It preserves living
units, casualties, wounds, experience, equipment, gold, round and resolved outcomes;
it never respawns the historical roster. The placeholder companion Meri becomes
Oren, including her existing progression. Sealed stones remain spent at their
mapped ritual sites. New fog and faction data are migration-owned frozen values.
Existing Mirror castle experiments return to the original terrain, and developer
castle cells receive their frozen complete-frame selections while retaining owners.
Current valley checkpoints and other maps retain their resolved state. This
conversion does not inspect neighbors or compose artwork.

## Version-eleven migration

The v11-to-v12 step corrects the valley opening. It removes the two unearned
starters only from unfinished turn-one battles where their published IDs, types,
positions and untouched stats still match. Recalled troops and units that acted,
earned experience, took damage or received equipment remain. Obsolete opening
dialogue is removed with its cursor adjusted; a present Oren supplies the monastery
lead once if the ritual has not started. Every constant belongs to this historical
conversion; it never queries current content or changes terrain.

## Version-twelve migration

The v12-to-v13 step corrects the published church reward to one enchanted mail
shirt. It removes the unintended chapel helm from ground inventory, active units,
veterans, story veterans and evacuated units in Lantern Road saves. Existing mail
keeps its mechanics and receives the corrected name and description; queued church
discovery text describes one item. Other battle progress remains intact. Constants
are frozen inside the conversion and do not query current content.

## Version-thirteen migration

The v13-to-v14 step installs three keep-side villages in ongoing valley battles,
using frozen coordinates and tile fields. Only the published plains sites change;
fog and all unrelated battle state are preserved. The villages begin enemy-owned,
except when a player unit already occupies one. Existing village ownership is not
reset. A player occupant on the newly capturable bell keep claims that tile too.
The obsolete courtyard garrison tag is removed so these units use ordinary AI.
Other maps pass through unchanged.

## Version-fourteen migration

The v14-to-v15 step assigns the living enemy Tollkeeper in Stage 1 to his
conditional hold group, replacing the old permanent leader post. It preserves
units, turns, completed events and all other battle progress. Other maps pass
through unchanged. The conversion uses only frozen assignment constants.

## Version-fifteen migration

The v15-to-v16 step corrects the three published keep-side villages in the Bell
valley from green-plain to mountain-rock. Only their floor changes; ownership,
fog, lights, units, turn, victory and all other progress are preserved. Coordinates
and prior values are frozen in the conversion. Other maps pass through unchanged.

## Version-sixteen migration

The v16-to-v17 step replaces castle connection/corner codes with explicit walls
and gates. It uses the frozen clockwise direction order and published gate rule,
removes the obsolete fields, and preserves ownership, fog, units and all battle
progress. It does not import current content or execute gameplay.

## Version-seventeen migration

The v17-to-v18 step retires dedicated castle gate terrain and its aperture fields.
Gate cells become ordinary castle courtyards with their existing wall directions;
the former gate edge stays open. All castle cells lose the obsolete gates array.
Ownership, fog, units, equipment, turns, victory and other battle progress remain
unchanged. The conversion uses only frozen version-seventeen field contracts.

## Version-eighteen migration

The v18-to-v19 step installs the northern hills and relocated Tollkeeper keep in
revision-four Villages saves, using a frozen tile delta. It advances that map to
revision five and preserves unit positions, health, equipment, gold, turns,
completed events and outcomes. Keep ownership transfers from the former keep,
unless a unit already occupies the new keep. Other maps pass through unchanged.

## Version-nineteen migration

The v19-to-v20 step bends the eastern Villages road northeast and moves its
departure from the sign at (18,5) to the northern pass at (17,0), advancing map
revision five to six. Frozen tile deltas widen and ease the approach to the pass
while preserving ownership, units, positions, health, gold, fog, completed events
and outcomes. Other maps pass through unchanged. Departure objectives and script
conditions use the northern pass through the current scenario definitions.

## Version-twenty migration

The v20-to-v21 step converts phase economies to faction economies. Existing
player/enemy balances and settlement owners remain unchanged; declared factions
receive empty independent treasuries. A frozen Villages revision-six tile delta
adds the marsh keep and funding villages and advances the map to revision seven.
Existing owners take precedence, followed by occupants on newly owned tiles.
A living Edrin gains the leader role and keep order; his faction receives its
explicit recruitment quotes. No units are spawned, revived or relocated, and no
gold is invented. The obsolete `group:swamp-dead` activation marker is retired
because that encounter is now unconditional. Health, items, progress, turns and
outcomes remain intact.

## Version-twenty-one migration

The v21-to-v22 step removes two added Villages settlements, restoring forest at
(13,7) and marsh at (16,10), and advances revision seven to eight. The conversion
uses frozen coordinates and terrain values. It preserves ownership, units, gold,
turns, equipment and campaign progress; existing armies are not disbanded and
treasuries are not reset to the new opening budgets. Other maps pass through.

## Version 23: tile-owned terrain lighting

Version 22 map emitters move onto their owning tiles using frozen light definitions
under `legacy/data/tile-lights-v22.json`. Remembered light keys change from list
index/coordinates to tile identity. Gameplay state and fog discovery are preserved;
rendered light positions are derived from tile coordinates.

## Version 24: explicit base factions

Faction rosters replace implicit player/enemy armies. The conversion appends frozen
base controller definitions only where absent, preserving saved units, treasuries,
existing faction definitions and campaign progress. Campaign recruitment continues
to use its resolved authored pools and completed recruitment events.

## Version 25: explicit diplomacy

The old engine treated player/enemy as hostile when neither faction declared
their relation. This conversion records that historical hostility on the existing
faction definitions. Explicit neutral, allied or hostile declarations on either
side remain unchanged. Units, gold and campaign progress are preserved.

## Version 26: campaign character identities

Frozen campaign registries map historical unit names and dialogue aliases to
character IDs. Units, veterans, evacuated units, companion-loss markers and saved
selector conditions retain their identity through conversion. Named dialogue
uses character IDs; current captions come from Rust inspection. Generic narrator
labels and existing prose/history remain unchanged. No gameplay commands run.
